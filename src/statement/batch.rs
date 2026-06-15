use std::sync::Arc;

use scylla::client::session::Session;
use scylla::statement::batch::{Batch, BatchType};
use scylla::value::CqlValue;

use crate::error::{driver_error, map_execution_error};
use crate::session::query_result::{query_result_from_scylla, QueryResult};
use crate::types::cql_json::json_to_bind_value;

#[napi(object)]
/// One statement entry added to a {@link ScyllaBatchStatement}.
pub struct BatchStatement {
  /// CQL query with `?` bind markers.
  pub query: String,
  /// Bind values for this statement.
  pub params: Option<Vec<serde_json::Value>>,
}

#[napi]
/// Builder for CQL batch statements (logged, unlogged, or counter).
pub struct ScyllaBatchStatement {
  session: Arc<Session>,
  batch_type: BatchType,
  statements: Vec<BatchStatement>,
}

#[napi]
impl ScyllaBatchStatement {
  pub fn new(session: Arc<Session>, batch_type: BatchType) -> Self {
    Self {
      session,
      batch_type,
      statements: Vec::new(),
    }
  }

  #[napi]
  /// Adds a statement to the batch. Returns `this` for chaining.
  pub fn add(&mut self, statement: BatchStatement) -> &Self {
    self.statements.push(statement);
    self
  }

  #[napi]
  /// Sends the batch to the cluster.
  pub async fn execute(&self) -> napi::Result<QueryResult> {
    let mut batch = Batch::new(self.batch_type);
    let mut all_values: Vec<Vec<Option<CqlValue>>> = Vec::new();

    for stmt in &self.statements {
      batch.append_statement(stmt.query.as_str());

      let values = if let Some(params) = &stmt.params {
        params
          .iter()
          .map(json_to_bind_value)
          .collect::<Result<Vec<_>, _>>()?
      } else {
        Vec::new()
      };

      all_values.push(values);
    }

    let values_refs: Vec<&[Option<CqlValue>]> =
      all_values.iter().map(|v| v.as_slice()).collect();

    let result = self
      .session
      .batch(&batch, values_refs)
      .await
      .map_err(map_execution_error)?;

    query_result_from_scylla(result)
  }
}

pub fn parse_batch_type(batch_type: &str) -> napi::Result<BatchType> {
  match batch_type.to_lowercase().as_str() {
    "logged" => Ok(BatchType::Logged),
    "unlogged" => Ok(BatchType::Unlogged),
    "counter" => Ok(BatchType::Counter),
    other => Err(driver_error(format!(
      "Unknown batch type '{}'. Use 'logged', 'unlogged', or 'counter'.",
      other
    ))),
  }
}
