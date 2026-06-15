use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::client::Compression;
use scylla::policies::timestamp_generator::{
  MonotonicTimestampGenerator, SimpleTimestampGenerator,
};

use crate::error::{driver_error, map_new_session_error};
use crate::policies::execution_profile::build_execution_profile_handle;
use crate::session::scylla_session::ScyllaSession;

use super::cluster_config::ClusterConfig;

#[napi(js_name = "Cluster")]
pub struct ScyllaCluster {
  config: ClusterConfig,
}

#[napi]
impl ScyllaCluster {
  #[napi(constructor)]
  pub fn new(config: ClusterConfig) -> napi::Result<Self> {
    if config.nodes.is_empty() {
      return Err(driver_error("At least one node is required"));
    }

    Ok(Self { config })
  }

  #[napi]
  pub async fn connect(&self, keyspace: Option<String>) -> napi::Result<ScyllaSession> {
    let mut builder = SessionBuilder::new().known_nodes(&self.config.nodes);

    if let Some(username) = &self.config.username {
      let password = self.config.password.as_deref().unwrap_or("");
      builder = builder.user(username, password);
    }

    if let Some(compression) = &self.config.compression {
      builder = match compression.as_str() {
        "lz4" => builder.compression(Some(Compression::Lz4)),
        "snappy" => builder.compression(Some(Compression::Snappy)),
        "none" => builder.compression(None),
        other => {
          return Err(driver_error(format!(
            "Unsupported compression '{}'. Use 'lz4', 'snappy', or 'none'.",
            other
          )));
        }
      };
    }

    if let Some(timeout_ms) = self.config.connection_timeout_ms {
      builder = builder.connection_timeout(Duration::from_millis(timeout_ms as u64));
    }

    if let Some(dc) = &self.config.local_datacenter {
      if let Some(rack) = &self.config.local_datacenter_rack {
        builder = builder.prefer_datacenter_and_rack(dc.clone(), rack.clone());
      } else {
        builder = builder.prefer_datacenter(dc.clone());
      }
    }

    if let Some(disallow) = self.config.disallow_shard_aware_port {
      builder = builder.disallow_shard_aware_port(disallow);
    }

    if let Some(nodelay) = self.config.tcp_nodelay {
      builder = builder.tcp_nodelay(nodelay);
    }

    if let Some(interval_ms) = self.config.tcp_keepalive_interval_ms {
      builder = builder.tcp_keepalive_interval(Duration::from_millis(interval_ms as u64));
    }

    if let Some(timeout_ms) = self.config.schema_agreement_timeout_ms {
      builder =
        builder.schema_agreement_timeout(Duration::from_millis(timeout_ms as u64));
    }

    if let Some(auto_await) = self.config.auto_await_schema_agreement {
      builder = builder.auto_await_schema_agreement(auto_await);
    }

    if let Some(profile_config) = &self.config.execution_profile {
      let handle = build_execution_profile_handle(profile_config)?;
      builder = builder.default_execution_profile_handle(handle);
    }

    if let Some(entries) = &self.config.address_translation {
      let mut map: HashMap<SocketAddr, SocketAddr> = HashMap::new();
      for entry in entries {
        let source: SocketAddr = entry
          .source_address
          .parse()
          .map_err(|e| driver_error(format!("Invalid source address '{}': {}", entry.source_address, e)))?;
        let target: SocketAddr = entry
          .target_address
          .parse()
          .map_err(|e| driver_error(format!("Invalid target address '{}': {}", entry.target_address, e)))?;
        map.insert(source, target);
      }
      builder = builder.address_translator(Arc::new(map));
    }

    if let Some(ts_gen) = &self.config.timestamp_generator {
      builder = match ts_gen.to_lowercase().as_str() {
        "simple" => {
          builder.timestamp_generator(Arc::new(SimpleTimestampGenerator::new()))
        }
        "monotonic" => {
          builder.timestamp_generator(Arc::new(MonotonicTimestampGenerator::new()))
        }
        other => {
          return Err(driver_error(format!(
            "Unknown timestamp generator '{}'. Use 'simple' or 'monotonic'.",
            other
          )));
        }
      };
    }

    if let Some(tls_config) = &self.config.tls {
      let tls_context = build_tls_context(tls_config)?;
      builder = builder.tls_context(Some(tls_context));
    }

    let effective_keyspace = keyspace.or_else(|| self.config.default_keyspace.clone());
    if let Some(keyspace_name) = effective_keyspace {
      builder = builder.use_keyspace(keyspace_name, true);
    }

    let session: Session = builder.build().await.map_err(map_new_session_error)?;

    Ok(ScyllaSession::new(session))
  }
}

fn build_tls_context(
  tls_config: &super::cluster_config::TlsConfig,
) -> napi::Result<Arc<rustls::ClientConfig>> {
  use rustls::ClientConfig;
  use std::io::BufReader;

  let mut root_store = rustls::RootCertStore::empty();

  if let Some(ca_path) = &tls_config.ca_filepath {
    let ca_file = std::fs::File::open(ca_path)
      .map_err(|e| driver_error(format!("Failed to open CA file '{}': {}", ca_path, e)))?;
    let mut reader = BufReader::new(ca_file);
    let certs = rustls_pemfile::certs(&mut reader)
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| driver_error(format!("Failed to read CA certs: {}", e)))?;
    for cert in certs {
      root_store
        .add(cert)
        .map_err(|e| driver_error(format!("Failed to add CA cert: {}", e)))?;
    }
  } else {
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
  }

  let config = ClientConfig::builder()
    .with_root_certificates(root_store)
    .with_no_client_auth();

  Ok(Arc::new(config))
}
