use crate::policies::execution_profile::ExecutionProfileConfig;

/// Per-statement execution options passed to {@link ScyllaSession.execute} and {@link ScyllaPreparedStatement.execute}.
#[napi(object)]
pub struct ExecuteOptions {
  /// Read/write consistency level (for example `local_quorum`, `one`).
  pub consistency: Option<String>,
  /// Serial consistency for lightweight transactions.
  pub serial_consistency: Option<String>,
  /// When true, enables server-side tracing for this statement.
  pub tracing: Option<bool>,
  /// Page size hint for paged reads.
  pub page_size: Option<u32>,
  /// Whether the statement is safe to retry on failure.
  pub is_idempotent: Option<bool>,
  /// Request timeout override in milliseconds.
  pub timeout_ms: Option<u32>,
  /// Execution profile override for this statement only.
  pub execution_profile: Option<ExecutionProfileConfig>,
  /// Client-side write timestamp in microseconds since Unix epoch.
  pub timestamp_micros: Option<i64>,
}
