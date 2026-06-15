use crate::policies::execution_profile::ExecutionProfileConfig;

#[napi(object)]
pub struct TlsConfig {
  pub ca_filepath: Option<String>,
  pub verify_identity: Option<bool>,
}

#[napi(object)]
pub struct AddressTranslationEntry {
  pub source_address: String,
  pub target_address: String,
}

#[napi(object)]
pub struct ClusterConfig {
  pub nodes: Vec<String>,
  pub username: Option<String>,
  pub password: Option<String>,
  pub compression: Option<String>,
  pub default_keyspace: Option<String>,
  pub connection_timeout_ms: Option<u32>,
  pub pool_size: Option<u32>,
  pub local_datacenter: Option<String>,
  pub local_datacenter_rack: Option<String>,
  pub tls: Option<TlsConfig>,
  pub disallow_shard_aware_port: Option<bool>,
  pub tcp_nodelay: Option<bool>,
  pub tcp_keepalive_interval_ms: Option<u32>,
  pub schema_agreement_timeout_ms: Option<u32>,
  pub auto_await_schema_agreement: Option<bool>,
  pub execution_profile: Option<ExecutionProfileConfig>,
  pub address_translation: Option<Vec<AddressTranslationEntry>>,
  pub timestamp_generator: Option<String>,
}
