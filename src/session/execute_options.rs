use crate::policies::execution_profile::ExecutionProfileConfig;

#[napi(object)]
pub struct ExecuteOptions {
  pub consistency: Option<String>,
  pub serial_consistency: Option<String>,
  pub tracing: Option<bool>,
  pub page_size: Option<u32>,
  pub is_idempotent: Option<bool>,
  pub timeout_ms: Option<u32>,
  pub execution_profile: Option<ExecutionProfileConfig>,
  pub timestamp_micros: Option<i64>,
}
