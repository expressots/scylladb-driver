#![deny(clippy::all)]
#![allow(dead_code)]

use scylla::frame::response::result::ColumnType;
use serde_json::{json, Value};

#[macro_use]
extern crate napi_derive;

#[napi]
struct Cluster {
  uri: String,
}

#[napi]
struct ScyllaSession {
  session: scylla::Session,
}

#[napi(object)]
struct ClusterConfig {
  pub nodes: Vec<String>,
}

#[napi]
impl Cluster {
  /// Object config is in the format:
  /// {
  ///     nodes: Array<string>,
  /// }
  #[napi(constructor)]
  pub fn new(object_config: ClusterConfig) -> Self {
    let nodes = object_config.nodes;

    let uri = nodes.get(0).expect("at least one node is required");

    Self {
      uri: uri.to_string(),
    }
  }

  #[napi]
  pub async fn connect(&self, _keyspace: Option<String>) -> ScyllaSession {
    ScyllaSession {
      session: scylla::SessionBuilder::new()
        .known_node(self.uri.as_str())
        .build()
        .await
        .unwrap(),
    }
  }
}

#[napi]
impl ScyllaSession {
  #[napi]
  pub async fn execute(&self, query: String) -> Value {
    let query_result = self.session.query(query, &[]).await.unwrap();

    // If no rows were found return an empty array
    if query_result.rows.is_none() {
      return json!([]);
    }

    let rows = query_result.rows.unwrap();
    let column_specs = query_result.col_specs;

    let mut result = json!([]);

    for row in rows {
      let mut row_object = serde_json::Map::new();

      for (i, column) in row.columns.iter().enumerate() {
        let column = column.clone().unwrap();
        let column_name = column_specs[i].name.clone();

        let column_value = match column_specs[i].typ {
          ColumnType::Ascii => Value::String(column.as_ascii().unwrap().to_string()),
          ColumnType::Text => Value::String(column.as_text().unwrap().to_string()),
          ColumnType::Uuid => Value::String(column.as_uuid().unwrap().to_string()),
          ColumnType::Int => {
            Value::Number(serde_json::Number::from_f64(column.as_int().unwrap() as f64).unwrap())
          }
          _ => "Not implemented".into(),
        };

        row_object.insert(column_name, column_value);
      }

      result
        .as_array_mut()
        .unwrap()
        .push(Value::Object(row_object));
    }

    result
  }
}
