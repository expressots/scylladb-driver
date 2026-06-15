use std::sync::Arc;

use scylla::client::session::Session;
use scylla::cluster::ClusterState;

use crate::error::driver_error;

#[napi(object)]
/// Keyspace metadata from the driver schema cache.
pub struct KeyspaceInfo {
  pub name: String,
  pub strategy: String,
  pub durable_writes: bool,
  pub tables: Vec<String>,
  pub views: Vec<String>,
  pub user_defined_types: Vec<String>,
}

#[napi(object)]
/// Column metadata within a table or view.
pub struct ColumnInfo {
  pub name: String,
  pub type_name: String,
  pub kind: String,
}

#[napi(object)]
/// Table metadata including partition and clustering keys.
pub struct TableInfo {
  pub name: String,
  pub partition_key: Vec<String>,
  pub clustering_key: Vec<String>,
  pub columns: Vec<ColumnInfo>,
  pub partitioner: Option<String>,
}

#[napi(object)]
/// Materialized view metadata.
pub struct MaterializedViewInfo {
  pub name: String,
  pub base_table_name: String,
  pub partition_key: Vec<String>,
  pub clustering_key: Vec<String>,
  pub columns: Vec<ColumnInfo>,
}

#[napi(object)]
/// Field metadata within a user-defined type.
pub struct UdtFieldInfo {
  pub name: String,
  pub type_name: String,
}

#[napi(object)]
/// User-defined type (UDT) metadata.
pub struct UserDefinedTypeInfo {
  pub name: String,
  pub keyspace: String,
  pub fields: Vec<UdtFieldInfo>,
}

pub fn get_keyspaces_from_session(session: &Session) -> Vec<KeyspaceInfo> {
  let state: Arc<ClusterState> = session.get_cluster_state();
  state
    .keyspaces_iter()
    .map(|(name, ks)| KeyspaceInfo {
      name: name.to_string(),
      strategy: format!("{:?}", ks.strategy),
      durable_writes: ks.durable_writes,
      tables: ks.tables.keys().cloned().collect(),
      views: ks.views.keys().cloned().collect(),
      user_defined_types: ks.user_defined_types.keys().cloned().collect(),
    })
    .collect()
}

pub fn get_table_from_session(
  session: &Session,
  keyspace: &str,
  table_name: &str,
) -> napi::Result<TableInfo> {
  let state: Arc<ClusterState> = session.get_cluster_state();
  for (ks_name, ks) in state.keyspaces_iter() {
    if ks_name == keyspace {
      if let Some(table) = ks.tables.get(table_name) {
        let columns: Vec<ColumnInfo> = table
          .columns
          .iter()
          .map(|(col_name, col)| ColumnInfo {
            name: col_name.clone(),
            type_name: format!("{:?}", col.typ),
            kind: format!("{:?}", col.kind),
          })
          .collect();

        return Ok(TableInfo {
          name: table_name.to_string(),
          partition_key: table.partition_key.clone(),
          clustering_key: table.clustering_key.clone(),
          columns,
          partitioner: table.partitioner.clone(),
        });
      }
      return Err(driver_error(format!(
        "Table '{}' not found in keyspace '{}'",
        table_name, keyspace
      )));
    }
  }
  Err(driver_error(format!("Keyspace '{}' not found", keyspace)))
}

pub fn get_materialized_view_from_session(
  session: &Session,
  keyspace: &str,
  view_name: &str,
) -> napi::Result<MaterializedViewInfo> {
  let state: Arc<ClusterState> = session.get_cluster_state();
  for (ks_name, ks) in state.keyspaces_iter() {
    if ks_name == keyspace {
      if let Some(view) = ks.views.get(view_name) {
        let columns: Vec<ColumnInfo> = view
          .view_metadata
          .columns
          .iter()
          .map(|(col_name, col)| ColumnInfo {
            name: col_name.clone(),
            type_name: format!("{:?}", col.typ),
            kind: format!("{:?}", col.kind),
          })
          .collect();

        return Ok(MaterializedViewInfo {
          name: view_name.to_string(),
          base_table_name: view.base_table_name.clone(),
          partition_key: view.view_metadata.partition_key.clone(),
          clustering_key: view.view_metadata.clustering_key.clone(),
          columns,
        });
      }
      return Err(driver_error(format!(
        "Materialized view '{}' not found in keyspace '{}'",
        view_name, keyspace
      )));
    }
  }
  Err(driver_error(format!("Keyspace '{}' not found", keyspace)))
}

pub fn get_udt_from_session(
  session: &Session,
  keyspace: &str,
  type_name: &str,
) -> napi::Result<UserDefinedTypeInfo> {
  let state: Arc<ClusterState> = session.get_cluster_state();
  for (ks_name, ks) in state.keyspaces_iter() {
    if ks_name == keyspace {
      if let Some(udt) = ks.user_defined_types.get(type_name) {
        let fields: Vec<UdtFieldInfo> = udt
          .field_types
          .iter()
          .map(|(fname, ftype)| UdtFieldInfo {
            name: fname.to_string(),
            type_name: format!("{:?}", ftype),
          })
          .collect();

        return Ok(UserDefinedTypeInfo {
          name: type_name.to_string(),
          keyspace: keyspace.to_string(),
          fields,
        });
      }
      return Err(driver_error(format!(
        "User defined type '{}' not found in keyspace '{}'",
        type_name, keyspace
      )));
    }
  }
  Err(driver_error(format!("Keyspace '{}' not found", keyspace)))
}
