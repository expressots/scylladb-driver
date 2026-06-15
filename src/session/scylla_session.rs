use std::sync::Arc;

use scylla::client::session::Session;
use scylla::statement::unprepared::Statement;
use scylla::statement::Consistency;
use scylla::statement::SerialConsistency;
use scylla::value::CqlValue;

use crate::error::{driver_error, map_execution_error};
use crate::observability::metrics::{extract_metrics, DriverMetrics};
use crate::observability::tracing_info::{get_tracing_info, QueryTracingInfo};
use crate::policies::execution_profile::build_execution_profile_handle;
use crate::schema::metadata::{
  get_keyspaces_from_session, get_materialized_view_from_session, get_table_from_session,
  get_udt_from_session, KeyspaceInfo, MaterializedViewInfo, TableInfo, UserDefinedTypeInfo,
};
use crate::session::execute_options::ExecuteOptions;
use crate::session::query_result::{
  AttemptHistoryInfo, PagedQueryResult, QueryResult, QueryWithHistory, RequestHistoryInfo,
  query_result_from_scylla,
};
use crate::statement::batch::{parse_batch_type, ScyllaBatchStatement};
use crate::statement::prepared::{prepare_statement, ScyllaPreparedStatement};
use crate::types::cql_json::json_to_bind_value;

#[napi]
/// An active connection to a ScyllaDB or Cassandra cluster.
///
/// Obtain a session from {@link Cluster.connect}. A session is cheap to share across your application.
pub struct ScyllaSession {
  session: Arc<Session>,
}

#[napi]
impl ScyllaSession {
  pub fn new(session: Session) -> Self {
    Self {
      session: Arc::new(session),
    }
  }

  pub fn session_ref(&self) -> &Session {
    &self.session
  }

  #[napi]
  /// Executes a CQL statement and returns all rows in a single response (unpaged).
  ///
  /// @param query - CQL query string with `?` placeholders for bind markers.
  /// @param parameters - Bind values as JSON-compatible values (see Rust driver data types docs).
  /// @param options - Optional per-statement settings (consistency, tracing, profile, etc.).
  pub async fn execute(
    &self,
    query: String,
    parameters: Option<Vec<serde_json::Value>>,
    options: Option<ExecuteOptions>,
  ) -> napi::Result<QueryResult> {
    let mut stmt = Statement::new(query);

    if let Some(opts) = &options {
      if let Some(consistency) = &opts.consistency {
        stmt.set_consistency(parse_consistency(consistency)?);
      }
      if let Some(serial) = &opts.serial_consistency {
        stmt.set_serial_consistency(Some(parse_serial_consistency(serial)?));
      }
      if let Some(tracing) = opts.tracing {
        stmt.set_tracing(tracing);
      }
      if let Some(idempotent) = opts.is_idempotent {
        stmt.set_is_idempotent(idempotent);
      }
      if let Some(profile_config) = &opts.execution_profile {
        let handle = build_execution_profile_handle(profile_config)?;
        stmt.set_execution_profile_handle(Some(handle));
      }
      if let Some(ts) = opts.timestamp_micros {
        stmt.set_timestamp(Some(ts));
      }
    }

    let query_result = if let Some(parameters) = parameters {
      let bind_values: Vec<Option<CqlValue>> = parameters
        .iter()
        .map(json_to_bind_value)
        .collect::<Result<_, _>>()?;

      self
        .session
        .query_unpaged(stmt, bind_values)
        .await
        .map_err(map_execution_error)?
    } else {
      self
        .session
        .query_unpaged(stmt, &[])
        .await
        .map_err(map_execution_error)?
    };

    query_result_from_scylla(query_result)
  }

  #[napi]
  /// Switches the session keyspace (`USE keyspace` equivalent).
  pub async fn use_keyspace(
    &self,
    keyspace: String,
    case_sensitive: Option<bool>,
  ) -> napi::Result<()> {
    self
      .session
      .use_keyspace(&keyspace, case_sensitive.unwrap_or(true))
      .await
      .map_err(|err| driver_error(format!("Failed to switch keyspace: {}", err)))
  }

  #[napi]
  /// Waits until cluster nodes agree on the current schema version. Returns the schema version UUID.
  pub async fn await_schema_agreement(&self) -> napi::Result<String> {
    let uuid = self
      .session
      .await_schema_agreement()
      .await
      .map_err(|err| driver_error(format!("Schema agreement failed: {}", err)))?;
    Ok(uuid.to_string())
  }

  #[napi]
  /// Returns the current schema version UUID if all nodes agree, or `null` if they do not.
  pub async fn check_schema_agreement(&self) -> napi::Result<Option<String>> {
    let result = self
      .session
      .check_schema_agreement()
      .await
      .map_err(|err| driver_error(format!("Schema agreement check failed: {}", err)))?;
    Ok(result.map(|uuid| uuid.to_string()))
  }

  #[napi]
  /// Prepares a CQL statement on the server for efficient repeated execution.
  pub async fn prepare(&self, query: String) -> napi::Result<ScyllaPreparedStatement> {
    let prepared = prepare_statement(&self.session, &query).await?;
    Ok(ScyllaPreparedStatement::new(self.session.clone(), prepared))
  }

  #[napi]
  /// Creates a batch builder. @param batchType - `logged`, `unlogged`, or `counter` (default: `logged`).
  pub fn batch(&self, batch_type: Option<String>) -> napi::Result<ScyllaBatchStatement> {
    let bt = match &batch_type {
      Some(t) => parse_batch_type(t)?,
      None => scylla::statement::batch::BatchType::Logged,
    };
    Ok(ScyllaBatchStatement::new(self.session.clone(), bt))
  }

  #[napi]
  /// Fetches a single page of results. Pass `nextPageToken` from a previous page to continue.
  pub async fn query_single_page(
    &self,
    query: String,
    parameters: Option<Vec<serde_json::Value>>,
    page_size: Option<u32>,
    paging_state: Option<Vec<u8>>,
  ) -> napi::Result<PagedQueryResult> {
    use scylla::response::PagingState;
    use scylla::statement::unprepared::Statement;
    use std::ops::ControlFlow;

    let mut stmt = Statement::new(query);
    stmt.set_page_size(page_size.unwrap_or(100) as i32);

    let ps = match paging_state {
      Some(bytes) => PagingState::new_from_raw_bytes(bytes),
      None => PagingState::start(),
    };

    let (result, next_paging_state) = if let Some(parameters) = parameters {
      let bind_values: Vec<Option<CqlValue>> = parameters
        .iter()
        .map(json_to_bind_value)
        .collect::<Result<_, _>>()?;

      self
        .session
        .query_single_page(stmt, bind_values, ps)
        .await
        .map_err(map_execution_error)?
    } else {
      self
        .session
        .query_single_page(stmt, &[], ps)
        .await
        .map_err(map_execution_error)?
    };

    let base = query_result_from_scylla(result)?;
    let next_page_token = match next_paging_state.into_paging_control_flow() {
      ControlFlow::Continue(state) => {
        state.as_bytes_slice().map(|arc| arc.to_vec())
      }
      ControlFlow::Break(()) => None,
    };

    Ok(PagedQueryResult {
      rows: base.rows,
      row_length: base.row_length,
      columns: base.columns,
      was_applied: base.was_applied,
      next_page_token,
      tracing_id: base.tracing_id,
    })
  }

  #[napi]
  /// Fetches all pages of a query automatically and returns the combined rows.
  pub async fn execute_paged(
    &self,
    query: String,
    parameters: Option<Vec<serde_json::Value>>,
    page_size: Option<u32>,
  ) -> napi::Result<QueryResult> {
    use scylla::statement::unprepared::Statement;

    let mut stmt = Statement::new(query);
    stmt.set_page_size(page_size.unwrap_or(5000) as i32);

    let rows_stream = if let Some(parameters) = parameters {
      let bind_values: Vec<Option<CqlValue>> = parameters
        .iter()
        .map(json_to_bind_value)
        .collect::<Result<_, _>>()?;

      self
        .session
        .query_iter(stmt, bind_values)
        .await
        .map_err(|e| driver_error(format!("Paged query failed: {}", e)))?
    } else {
      self
        .session
        .query_iter(stmt, &[])
        .await
        .map_err(|e| driver_error(format!("Paged query failed: {}", e)))?
    };

    use scylla::value::Row;
    use futures::StreamExt;
    use crate::types::cql_json::cql_value_to_json;

    let col_specs: Vec<crate::session::query_result::ColumnSpec> = rows_stream
      .column_specs()
      .iter()
      .map(|spec| crate::session::query_result::ColumnSpec {
        name: spec.name().to_string(),
        type_name: format!("{:?}", spec.typ()),
      })
      .collect();

    let col_names: Vec<String> = col_specs.iter().map(|s| s.name.clone()).collect();

    let mut all_rows: Vec<serde_json::Value> = Vec::new();
    let mut stream = rows_stream.rows_stream::<Row>().map_err(|e| {
      driver_error(format!("Failed to create row stream: {}", e))
    })?;

    while let Some(row_result) = stream.next().await {
      let row = row_result.map_err(|e| {
        driver_error(format!("Failed to read row: {}", e))
      })?;
      let mut row_object = serde_json::Map::new();
      for (index, column) in row.columns.iter().enumerate() {
        let col_name = col_names
          .get(index)
          .cloned()
          .unwrap_or_else(|| format!("column_{}", index));
        let col_value = match column {
          Some(value) => cql_value_to_json(value),
          None => serde_json::Value::Null,
        };
        row_object.insert(col_name, col_value);
      }
      all_rows.push(serde_json::Value::Object(row_object));
    }

    Ok(QueryResult {
      row_length: all_rows.len() as u32,
      rows: all_rows,
      columns: col_specs,
      was_applied: None,
      tracing_id: None,
    })
  }

  #[napi]
  /// Returns metadata for all keyspaces known to the driver.
  pub fn get_keyspaces(&self) -> Vec<KeyspaceInfo> {
    get_keyspaces_from_session(&self.session)
  }

  #[napi]
  /// Returns table metadata for the given keyspace and table name.
  pub fn get_table(&self, keyspace: String, table: String) -> napi::Result<TableInfo> {
    get_table_from_session(&self.session, &keyspace, &table)
  }

  #[napi]
  /// Returns materialized view metadata.
  pub fn get_materialized_view(
    &self,
    keyspace: String,
    view_name: String,
  ) -> napi::Result<MaterializedViewInfo> {
    get_materialized_view_from_session(&self.session, &keyspace, &view_name)
  }

  #[napi]
  /// Returns user-defined type (UDT) metadata.
  pub fn get_user_defined_type(
    &self,
    keyspace: String,
    type_name: String,
  ) -> napi::Result<UserDefinedTypeInfo> {
    get_udt_from_session(&self.session, &keyspace, &type_name)
  }

  #[napi]
  /// Refreshes cluster metadata from the server (keyspaces, tables, types, views).
  pub async fn refresh_metadata(&self) -> napi::Result<()> {
    self
      .session
      .refresh_metadata()
      .await
      .map_err(|err| driver_error(format!("Failed to refresh metadata: {}", err)))
  }

  #[napi]
  /// Executes a query and returns the result together with retry/speculative execution history.
  pub async fn execute_with_history(
    &self,
    query: String,
    parameters: Option<Vec<serde_json::Value>>,
  ) -> napi::Result<QueryWithHistory> {
    use scylla::observability::history::HistoryCollector;
    use std::sync::Arc;

    let history_collector = Arc::new(HistoryCollector::new());
    let mut stmt = Statement::new(query);
    stmt.set_history_listener(history_collector.clone());

    let query_result = if let Some(parameters) = parameters {
      let bind_values: Vec<Option<CqlValue>> = parameters
        .iter()
        .map(json_to_bind_value)
        .collect::<Result<_, _>>()?;
      self
        .session
        .query_unpaged(stmt, bind_values)
        .await
        .map_err(map_execution_error)?
    } else {
      self
        .session
        .query_unpaged(stmt, &[])
        .await
        .map_err(map_execution_error)?
    };

    let base = query_result_from_scylla(query_result)?;
    let structured = history_collector.clone_structured_history();

    let requests: Vec<RequestHistoryInfo> = structured
      .requests
      .iter()
      .map(|req| {
        let attempts: Vec<AttemptHistoryInfo> = req
          .non_speculative_fiber
          .attempts
          .iter()
          .map(|a| AttemptHistoryInfo {
            node_address: a.node_addr.to_string(),
            success: a.result.as_ref().map(|r| matches!(r, scylla::observability::history::AttemptResult::Success(_))),
            error: a.result.as_ref().and_then(|r| match r {
              scylla::observability::history::AttemptResult::Error(_, err, _) => Some(format!("{}", err)),
              _ => None,
            }),
          })
          .collect();
        let speculative_attempts: Vec<AttemptHistoryInfo> = req
          .speculative_fibers
          .iter()
          .flat_map(|fiber| fiber.attempts.iter())
          .map(|a| AttemptHistoryInfo {
            node_address: a.node_addr.to_string(),
            success: a.result.as_ref().map(|r| matches!(r, scylla::observability::history::AttemptResult::Success(_))),
            error: a.result.as_ref().and_then(|r| match r {
              scylla::observability::history::AttemptResult::Error(_, err, _) => Some(format!("{}", err)),
              _ => None,
            }),
          })
          .collect();
        let success = req.result.as_ref().map(|r| {
          matches!(r, scylla::observability::history::RequestHistoryResult::Success(_))
        });
        RequestHistoryInfo {
          attempts,
          speculative_attempts,
          success,
        }
      })
      .collect();

    Ok(QueryWithHistory {
      result: base,
      history: requests,
    })
  }

  #[napi]
  /// Returns driver metrics (query counts, latencies, connection stats).
  pub fn get_metrics(&self) -> DriverMetrics {
    extract_metrics(&self.session.get_metrics())
  }

  #[napi]
  /// Fetches detailed tracing information for a tracing session id from {@link QueryResult.tracingId}.
  pub async fn get_tracing_info(&self, tracing_id: String) -> napi::Result<QueryTracingInfo> {
    get_tracing_info(&self.session, &tracing_id).await
  }
}

pub fn parse_consistency(value: &str) -> napi::Result<Consistency> {
  match value.to_lowercase().as_str() {
    "any" => Ok(Consistency::Any),
    "one" => Ok(Consistency::One),
    "two" => Ok(Consistency::Two),
    "three" => Ok(Consistency::Three),
    "quorum" => Ok(Consistency::Quorum),
    "all" => Ok(Consistency::All),
    "local_quorum" | "localquorum" => Ok(Consistency::LocalQuorum),
    "each_quorum" | "eachquorum" => Ok(Consistency::EachQuorum),
    "local_one" | "localone" => Ok(Consistency::LocalOne),
    other => Err(driver_error(format!(
      "Unknown consistency level '{}'. Use one of: any, one, two, three, quorum, all, local_quorum, each_quorum, local_one.",
      other
    ))),
  }
}

pub fn parse_serial_consistency(value: &str) -> napi::Result<SerialConsistency> {
  match value.to_lowercase().as_str() {
    "serial" => Ok(SerialConsistency::Serial),
    "local_serial" | "localserial" => Ok(SerialConsistency::LocalSerial),
    other => Err(driver_error(format!(
      "Unknown serial consistency '{}'. Use 'serial' or 'local_serial'.",
      other
    ))),
  }
}
