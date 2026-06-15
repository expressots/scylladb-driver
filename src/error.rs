use napi::Status;

pub fn driver_error(message: impl Into<String>) -> napi::Error {
  napi::Error::new(Status::GenericFailure, message.into())
}

pub fn map_execution_error(err: scylla::errors::ExecutionError) -> napi::Error {
  driver_error(format!("Query execution failed: {}", err))
}

pub fn map_new_session_error(err: scylla::errors::NewSessionError) -> napi::Error {
  driver_error(format!("Failed to connect to cluster: {}", err))
}
