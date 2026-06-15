use crate::policies::execution_profile::ExecutionProfileConfig;

/// TLS settings for encrypted connections to the cluster.
/// See [TLS in the Rust driver](https://rust-driver.docs.scylladb.com/stable/connecting/tls.html).
#[napi(object)]
pub struct TlsConfig {
  /// Path to a PEM file with trusted CA certificates. Uses system roots when omitted.
  pub ca_filepath: Option<String>,
  /// Whether to verify the server certificate hostname. Defaults to driver behavior when omitted.
  pub verify_identity: Option<bool>,
}

/// Maps a contact-point address to a different address (private networking / client routes).
/// See [Client routes](https://rust-driver.docs.scylladb.com/stable/connecting/client-routes.html).
#[napi(object)]
pub struct AddressTranslationEntry {
  /// Address returned by the cluster metadata (for example `10.0.0.5:9042`).
  pub source_address: String,
  /// Address the driver should connect to instead (for example `192.168.1.5:9042`).
  pub target_address: String,
}

/// Connection and session defaults for a ScyllaDB or Cassandra cluster.
/// See [Connecting to the cluster](https://rust-driver.docs.scylladb.com/stable/connecting/connecting.html).
#[napi(object)]
pub struct ClusterConfig {
  /// Contact points as `host:port` strings (for example `127.0.0.1:9042`).
  pub nodes: Vec<String>,
  /// Username for password authentication.
  pub username: Option<String>,
  /// Password for password authentication.
  pub password: Option<String>,
  /// Request compression: `lz4`, `snappy`, or `none`.
  pub compression: Option<String>,
  /// Keyspace used automatically when connecting.
  pub default_keyspace: Option<String>,
  /// TCP connection timeout in milliseconds.
  pub connection_timeout_ms: Option<u32>,
  /// Maximum number of connections per shard (reserved for future use).
  pub pool_size: Option<u32>,
  /// Local datacenter name for datacenter-aware load balancing.
  pub local_datacenter: Option<String>,
  /// Rack name within the local datacenter.
  pub local_datacenter_rack: Option<String>,
  /// TLS configuration. Omit to connect without TLS.
  pub tls: Option<TlsConfig>,
  /// When true, disables shard-aware port connections.
  pub disallow_shard_aware_port: Option<bool>,
  /// TCP_NODELAY socket option.
  pub tcp_nodelay: Option<bool>,
  /// TCP keepalive interval in milliseconds.
  pub tcp_keepalive_interval_ms: Option<u32>,
  /// Timeout for schema agreement waits in milliseconds.
  pub schema_agreement_timeout_ms: Option<u32>,
  /// Whether the driver waits for schema agreement after DDL automatically.
  pub auto_await_schema_agreement: Option<bool>,
  /// Default execution profile applied to statements unless overridden.
  pub execution_profile: Option<ExecutionProfileConfig>,
  /// Address translation table for private-network routing.
  pub address_translation: Option<Vec<AddressTranslationEntry>>,
  /// Client-side timestamp generator: `simple` or `monotonic`.
  pub timestamp_generator: Option<String>,
}
