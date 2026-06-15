use tracing_subscriber::EnvFilter;

#[napi]
pub fn init_logging(level: Option<String>) {
  let filter = level.unwrap_or_else(|| "scylla=info".to_string());
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(filter))
    .with_target(true)
    .with_ansi(false)
    .try_init();
}
