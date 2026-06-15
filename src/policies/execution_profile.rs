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

#[napi(object)]
pub struct SpeculativeExecutionConfig {
  pub max_retry_count: u32,
  pub retry_interval_ms: u32,
}

#[napi(object)]
pub struct PercentileSpeculativeConfig {
  pub max_retry_count: u32,
  pub percentile: f64,
}

#[napi(object)]
pub struct ExecutionProfileConfig {
  pub consistency: Option<String>,
  pub serial_consistency: Option<String>,
  pub request_timeout_ms: Option<u32>,
  pub retry_policy: Option<String>,
  pub speculative_execution: Option<SpeculativeExecutionConfig>,
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
