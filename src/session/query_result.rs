use scylla::response::query_result::QueryResult as ScyllaQueryResult;
use scylla::value::{CqlValue, Row};

use crate::error::driver_error;
use crate::types::cql_json::cql_value_to_json;

/// Metadata for one column in a query result set.
#[napi(object)]
pub struct ColumnSpec {
  /// Column name.
  pub name: String,
  /// CQL type name as reported by the server.
  pub type_name: String,
}

/// Result of a CQL statement that returns rows or an LWT outcome.
#[napi(object)]
pub struct QueryResult {
  /// Result rows as plain objects keyed by column name.
  pub rows: Vec<serde_json::Value>,
  /// Number of rows in this result page.
  pub row_length: u32,
  /// Column metadata for the result set.
  pub columns: Vec<ColumnSpec>,
  /// For lightweight transactions: whether the condition was applied. Undefined when not an LWT result.
  pub was_applied: Option<bool>,
  /// Tracing session id when tracing was enabled. Undefined when tracing was not requested.
  pub tracing_id: Option<String>,
}

/// One page of a manually paged query from {@link ScyllaSession.querySinglePage}.
#[napi(object)]
pub struct PagedQueryResult {
  /// Rows in this page.
  pub rows: Vec<serde_json::Value>,
  /// Number of rows in this page.
  pub row_length: u32,
  /// Column metadata for the result set.
  pub columns: Vec<ColumnSpec>,
  /// LWT outcome when applicable.
  pub was_applied: Option<bool>,
  /// Opaque token for the next page. Undefined when there are no more pages.
  pub next_page_token: Option<Vec<u8>>,
  /// Tracing session id when tracing was enabled.
  pub tracing_id: Option<String>,
}

/// One connection attempt recorded in query execution history.
#[napi(object)]
pub struct AttemptHistoryInfo {
  /// Node address used for this attempt.
  pub node_address: String,
  /// Whether the attempt succeeded.
  pub success: Option<bool>,
  /// Error message when the attempt failed.
  pub error: Option<String>,
}

/// Retry and speculative-execution history for one request.
#[napi(object)]
pub struct RequestHistoryInfo {
  /// Non-speculative retry attempts.
  pub attempts: Vec<AttemptHistoryInfo>,
  /// Speculative execution attempts.
  pub speculative_attempts: Vec<AttemptHistoryInfo>,
  /// Whether the overall request succeeded.
  pub success: Option<bool>,
}

/// Query result paired with execution history from {@link ScyllaSession.executeWithHistory}.
#[napi(object)]
pub struct QueryWithHistory {
  /// The query result.
  pub result: QueryResult,
  /// Per-request execution history entries.
  pub history: Vec<RequestHistoryInfo>,
}

pub fn query_result_from_scylla(result: ScyllaQueryResult) -> Result<QueryResult, napi::Error> {
  let tracing_id = result.tracing_id().map(|id| id.to_string());
  match result.into_rows_result() {
    Ok(rows_result) => {
      let columns: Vec<ColumnSpec> = rows_result
        .column_specs()
        .iter()
        .map(|spec| ColumnSpec {
          name: spec.name().to_string(),
          type_name: format!("{:?}", spec.typ()),
        })
        .collect();

      let mut rows = Vec::new();
      let mut was_applied = None;

      for row in rows_result
        .rows::<Row>()
        .map_err(|err| driver_error(format!("Failed to deserialize rows: {}", err)))?
      {
        let row = row.map_err(|err| driver_error(format!("Failed to deserialize row: {}", err)))?;
        let mut row_object = serde_json::Map::new();

        for (index, column) in row.columns.iter().enumerate() {
          let column_name = columns
            .get(index)
            .map(|spec| spec.name.clone())
            .unwrap_or_else(|| format!("column_{}", index));

          if column_name == "[applied]" {
            if let Some(CqlValue::Boolean(applied)) = column {
              was_applied = Some(*applied);
            }
          }

          let column_value = match column {
            Some(value) => cql_value_to_json(value),
            None => serde_json::Value::Null,
          };

          row_object.insert(column_name, column_value);
        }

        rows.push(serde_json::Value::Object(row_object));
      }

      Ok(QueryResult {
        row_length: rows.len() as u32,
        rows,
        columns,
        was_applied,
        tracing_id,
      })
    }
    Err(err) => {
      if let scylla::response::query_result::IntoRowsResultError::ResultNotRows(_) = err {
        return Ok(QueryResult {
          rows: Vec::new(),
          row_length: 0,
          columns: Vec::new(),
          was_applied: None,
          tracing_id,
        });
      }

      Err(driver_error(format!("Failed to read query result: {}", err)))
    }
  }
}