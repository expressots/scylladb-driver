use scylla::frame::response::result::ColumnType;

#[napi]
pub struct ScyllaSession {
  session: scylla::Session,
}

#[napi]
impl ScyllaSession {
  pub fn new(session: scylla::Session) -> Self {
    Self { session }
  }

  #[napi]
  pub async fn execute(&self, query: String) -> serde_json::Value {
    let query_result = self.session.query(query, &[]).await.unwrap();

    // If no rows were found return an empty array
    if query_result.rows.is_none() {
      return serde_json::json!([]);
    }

    let rows = query_result.rows.unwrap();
    let column_specs = query_result.col_specs;

    let mut result = serde_json::json!([]);

    for row in rows {
      let mut row_object = serde_json::Map::new();

      for (i, column) in row.columns.iter().enumerate() {
        let column = column.clone().unwrap();
        let column_name = column_specs[i].name.clone();

        let column_value = match column_specs[i].typ {
          ColumnType::Ascii => serde_json::Value::String(column.as_ascii().unwrap().to_string()),
          ColumnType::Text => serde_json::Value::String(column.as_text().unwrap().to_string()),
          ColumnType::Uuid => serde_json::Value::String(column.as_uuid().unwrap().to_string()),
          ColumnType::Int => serde_json::Value::Number(
            serde_json::Number::from_f64(column.as_int().unwrap() as f64).unwrap(),
          ),
          _ => "Not implemented".into(),
        };

        row_object.insert(column_name, column_value);
      }

      result
        .as_array_mut()
        .unwrap()
        .push(serde_json::Value::Object(row_object));
    }

    result
  }
}
