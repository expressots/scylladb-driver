use std::sync::Arc;
use std::time::Duration;

use scylla::client::execution_profile::{ExecutionProfile, ExecutionProfileHandle};
use scylla::policies::retry::{
  DefaultRetryPolicy, DowngradingConsistencyRetryPolicy, FallthroughRetryPolicy,
};
use scylla::policies::speculative_execution::{
  PercentileSpeculativeExecutionPolicy, SimpleSpeculativeExecutionPolicy,
};

use crate::error::driver_error;
use crate::session::scylla_session::{parse_consistency, parse_serial_consistency};

/// Simple speculative execution: retry on additional nodes after a fixed delay.
/// See [Simple speculative execution](https://rust-driver.docs.scylladb.com/stable/speculative_execution/simple-speculative-execution.html).
#[napi(object)]
pub struct SpeculativeExecutionConfig {
  /// Maximum number of speculative attempts.
  pub max_retry_count: u32,
  /// Delay between speculative attempts in milliseconds.
  pub retry_interval_ms: u32,
}

/// Percentile-based speculative execution policy configuration.
/// See [Percentile speculative execution](https://rust-driver.docs.scylladb.com/stable/speculative_execution/percentile-speculative-execution.html).
#[napi(object)]
pub struct PercentileSpeculativeConfig {
  /// Maximum number of speculative attempts.
  pub max_retry_count: u32,
  /// Latency percentile threshold (0.0 to 100.0).
  pub percentile: f64,
}

/// Groups execution settings (consistency, timeout, retry, speculative execution).
/// See [Execution profiles](https://rust-driver.docs.scylladb.com/stable/execution_profiles/execution_profiles.html).
#[napi(object)]
pub struct ExecutionProfileConfig {
  /// Default consistency: `one`, `quorum`, `local_quorum`, `local_one`, etc.
  pub consistency: Option<String>,
  /// Serial consistency for lightweight transactions: `serial` or `local_serial`.
  pub serial_consistency: Option<String>,
  /// Per-statement request timeout in milliseconds.
  pub request_timeout_ms: Option<u32>,
  /// Retry policy: `default`, `downgrading_consistency`, or `fallthrough`.
  pub retry_policy: Option<String>,
  /// Fixed-delay speculative execution policy.
  pub speculative_execution: Option<SpeculativeExecutionConfig>,
  /// Percentile-based speculative execution policy (mutually exclusive with `speculativeExecution`).
  pub percentile_speculative_execution: Option<PercentileSpeculativeConfig>,
}

pub fn build_execution_profile(
  config: &ExecutionProfileConfig,
) -> napi::Result<ExecutionProfile> {
  let mut builder = ExecutionProfile::builder();

  if let Some(consistency) = &config.consistency {
    builder = builder.consistency(parse_consistency(consistency)?);
  }

  if let Some(serial) = &config.serial_consistency {
    builder = builder.serial_consistency(Some(parse_serial_consistency(serial)?));
  }

  if let Some(timeout_ms) = config.request_timeout_ms {
    builder = builder.request_timeout(Some(Duration::from_millis(timeout_ms as u64)));
  }

  if let Some(policy_name) = &config.retry_policy {
    builder = match policy_name.to_lowercase().as_str() {
      "default" => builder.retry_policy(Arc::new(DefaultRetryPolicy::new())),
      "downgrading_consistency" | "downgradingconsistency" => {
        builder.retry_policy(Arc::new(DowngradingConsistencyRetryPolicy::new()))
      }
      "fallthrough" => builder.retry_policy(Arc::new(FallthroughRetryPolicy::new())),
      other => {
        return Err(driver_error(format!(
          "Unknown retry policy '{}'. Use 'default', 'downgrading_consistency', or 'fallthrough'.",
          other
        )));
      }
    };
  }

  if let Some(spec_config) = &config.speculative_execution {
    builder = builder.speculative_execution_policy(Some(Arc::new(
      SimpleSpeculativeExecutionPolicy {
        max_retry_count: spec_config.max_retry_count as usize,
        retry_interval: Duration::from_millis(spec_config.retry_interval_ms as u64),
      },
    )));
  } else if let Some(pct_config) = &config.percentile_speculative_execution {
    builder = builder.speculative_execution_policy(Some(Arc::new(
      PercentileSpeculativeExecutionPolicy {
        max_retry_count: pct_config.max_retry_count as usize,
        percentile: pct_config.percentile,
      },
    )));
  }

  Ok(builder.build())
}

pub fn build_execution_profile_handle(
  config: &ExecutionProfileConfig,
) -> napi::Result<ExecutionProfileHandle> {
  Ok(build_execution_profile(config)?.into_handle())
}
