use std::sync::Arc;

use scylla::observability::metrics::Metrics;

#[napi(object)]
/// Driver performance and connection statistics.
/// See [Driver metrics](https://rust-driver.docs.scylladb.com/stable/metrics/metrics.html).
pub struct DriverMetrics {
  /// Total number of queries executed.
  pub queries_num: u32,
  /// Total number of query errors.
  pub errors_num: u32,
  /// Total number of paged query iterators started.
  pub queries_iter_num: u32,
  /// Total number of paged query iterator errors.
  pub errors_iter_num: u32,
  /// Total number of query retries.
  pub retries_num: u32,
  /// Average query latency in milliseconds.
  pub latency_avg_ms: Option<u32>,
  /// 99th percentile latency in milliseconds.
  pub latency_p99_ms: Option<u32>,
  /// 95th percentile latency in milliseconds.
  pub latency_p95_ms: Option<u32>,
  /// Current total open connections.
  pub total_connections: u32,
  /// Number of connection timeouts.
  pub connection_timeouts: u32,
  /// Number of request timeouts.
  pub request_timeouts: u32,
}

pub fn extract_metrics(metrics: &Arc<Metrics>) -> DriverMetrics {
  DriverMetrics {
    queries_num: metrics.get_queries_num() as u32,
    errors_num: metrics.get_errors_num() as u32,
    queries_iter_num: metrics.get_queries_iter_num() as u32,
    errors_iter_num: metrics.get_errors_iter_num() as u32,
    retries_num: metrics.get_retries_num() as u32,
    latency_avg_ms: metrics.get_latency_avg_ms().ok().map(|v| v as u32),
    latency_p99_ms: metrics
      .get_latency_percentile_ms(99.0)
      .ok()
      .map(|v| v as u32),
    latency_p95_ms: metrics
      .get_latency_percentile_ms(95.0)
      .ok()
      .map(|v| v as u32),
    total_connections: metrics.get_total_connections() as u32,
    connection_timeouts: metrics.get_connection_timeouts() as u32,
    request_timeouts: metrics.get_request_timeouts() as u32,
  }
}
