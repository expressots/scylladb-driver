use std::sync::Arc;

use scylla::client::session::Session;
use scylla::observability::tracing::TracingInfo;
use uuid::Uuid;

use crate::error::driver_error;

#[napi(object)]
/// One event in a query trace timeline.
pub struct TracingEvent {
  pub event_id: String,
  pub activity: Option<String>,
  pub source: Option<String>,
  pub source_elapsed_ms: Option<i32>,
  pub thread: Option<String>,
}

#[napi(object)]
/// Detailed tracing information for a traced query.
/// See [Query tracing](https://rust-driver.docs.scylladb.com/stable/tracing/tracing.html).
pub struct QueryTracingInfo {
  pub client: Option<String>,
  pub command: Option<String>,
  pub coordinator: Option<String>,
  pub duration_us: Option<i32>,
  pub request: Option<String>,
  pub events: Vec<TracingEvent>,
}

pub async fn get_tracing_info(
  session: &Arc<Session>,
  tracing_id: &str,
) -> napi::Result<QueryTracingInfo> {
  let uuid = Uuid::parse_str(tracing_id)
    .map_err(|e| driver_error(format!("Invalid tracing UUID: {}", e)))?;

  let info: TracingInfo = session
    .get_tracing_info(&uuid)
    .await
    .map_err(|e| driver_error(format!("Failed to get tracing info: {}", e)))?;

  let events: Vec<TracingEvent> = info
    .events
    .iter()
    .map(|ev| TracingEvent {
      event_id: ev.event_id.to_string(),
      activity: ev.activity.clone(),
      source: ev.source.map(|ip| ip.to_string()),
      source_elapsed_ms: ev.source_elapsed,
      thread: ev.thread.clone(),
    })
    .collect();

  Ok(QueryTracingInfo {
    client: info.client.map(|ip| ip.to_string()),
    command: info.command,
    coordinator: info.coordinator.map(|ip| ip.to_string()),
    duration_us: info.duration,
    request: info.request,
    events,
  })
}
