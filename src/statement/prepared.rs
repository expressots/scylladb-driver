use std::sync::Arc;

use scylla::client::session::Session;
use scylla::statement::prepared::PreparedStatement;
use scylla::value::CqlValue;

use crate::error::{driver_error, map_execution_error};
use crate::session::execute_options::ExecuteOptions;
use crate::session::query_result::{query_result_from_scylla, QueryResult};
use crate::session::scylla_session::{parse_consistency, parse_serial_consistency};
use crate::types::cql_json::json_to_bind_value;

#[napi]
/// A server-prepared CQL statement for efficient repeated execution.
pub struct ScyllaPreparedStatement {
  session: Arc<Session>,
  prepared: PreparedStatement,
}

#[napi]
impl ScyllaPreparedStatement {
  pub fn new(session: Arc<Session>, prepared: PreparedStatement) -> Self {
    Self { session, prepared }
  }

  #[napi]
  /// Returns the original CQL text used to prepare this statement.
  pub fn get_query(&self) -> String {
    self.prepared.get_statement().to_string()
  }

  #[napi]
  /// Executes the prepared statement with optional bind values and execution options.
  pub async fn execute(
    &self,
    parameters: Option<Vec<serde_json::Value>>,
    options: Option<ExecuteOptions>,
  ) -> napi::Result<QueryResult> {
    let mut stmt = self.prepared.clone();

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
    }

    let query_result = if let Some(parameters) = parameters {
      let bind_values: Vec<Option<CqlValue>> = parameters
        .iter()
        .map(json_to_bind_value)
        .collect::<Result<_, _>>()?;

      self
        .session
        .execute_unpaged(&stmt, bind_values)
        .await
        .map_err(map_execution_error)?
    } else {
      self
        .session
        .execute_unpaged(&stmt, &[])
        .await
        .map_err(map_execution_error)?
    };

    query_result_from_scylla(query_result)
  }

  #[napi]
  /// Sets the default consistency for this prepared statement.
  pub fn set_consistency(&mut self, consistency: String) -> napi::Result<()> {
    self
      .prepared
      .set_consistency(parse_consistency(&consistency)?);
    Ok(())
  }

  #[napi]
  /// Sets the serial consistency for lightweight transactions on this statement.
  pub fn set_serial_consistency(
    &mut self,
    serial_consistency: String,
  ) -> napi::Result<()> {
    self
      .prepared
      .set_serial_consistency(Some(parse_serial_consistency(&serial_consistency)?));
    Ok(())
  }

  #[napi]
  /// Marks whether this statement is safe to retry.
  pub fn set_idempotent(&mut self, is_idempotent: bool) {
    self.prepared.set_is_idempotent(is_idempotent);
  }

  #[napi]
  /// Enables or disables server-side tracing for this prepared statement.
  pub fn set_tracing(&mut self, tracing: bool) {
    self.prepared.set_tracing(tracing);
  }
}

pub async fn prepare_statement(
  session: &Arc<Session>,
  query: &str,
) -> napi::Result<PreparedStatement> {
  session
    .prepare(query.to_string())
    .await
    .map_err(|err| driver_error(format!("Failed to prepare statement: {}", err)))
}
