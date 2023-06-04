use crate::session::scylla_session::ScyllaSession;

use super::cluster_config::ClusterConfig;

#[napi]
struct ScyllaCluster {
  uri: String,
}

#[napi]
impl ScyllaCluster {
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
    ScyllaSession::new(
      scylla::SessionBuilder::new()
        .known_node(self.uri.as_str())
        .build()
        .await
        .unwrap(),
    )
  }
}
