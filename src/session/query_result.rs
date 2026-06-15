use scylla::response::query_result::QueryResult as ScyllaQueryResult;
use scylla::value::{CqlValue, Row};

use crate::error::driver_error;
use crate::types::cql_json::cql_value_to_json;

#[napi(object)]
pub struct ColumnSpec {
  pub name: String,
  pub type_name: String,
}

#[napi(object)]
pub struct QueryResult {
  pub rows: Vec<serde_json::Value>,
  pub row_length: u32,
  pub columns: Vec<ColumnSpec>,
  pub was_applied: Option<bool>,
  pub tracing_id: Option<String>,
}

#[napi(object)]
pub struct PagedQueryResult {
  pub rows: Vec<serde_json::Value>,
  pub row_length: u32,
  pub columns: Vec<ColumnSpec>,
  pub was_applied: Option<bool>,
  pub next_page_token: Option<Vec<u8>>,
  pub tracing_id: Option<String>,
}

#[napi(object)]
pub struct AttemptHistoryInfo {
  pub node_address: String,
  pub success: Option<bool>,
  pub error: Option<String>,
}

#[napi(object)]
pub struct RequestHistoryInfo {
  pub attempts: Vec<AttemptHistoryInfo>,
  pub speculative_attempts: Vec<AttemptHistoryInfo>,
  pub success: Option<bool>,
}

#[napi(object)]
pub struct QueryWithHistory {
  pub result: QueryResult,
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
