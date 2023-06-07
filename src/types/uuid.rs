use napi::Result as NapiResult;

/// `Uuid` and `Timeuuid` are represented through this type.
#[napi]
struct Uuid {
  value: uuid::Uuid,
}

#[napi]
impl Uuid {
  /// Generates a random UUID v4.
  #[napi]
  pub fn random_uuidv4() -> Self {
    Self {
      value: uuid::Uuid::new_v4(),
    }
  }

  /// Parses a UUID from a string. It may fail if the string is not a valid UUID.
  #[napi]
  pub fn from_string(str: String) -> NapiResult<String> {
    let uuid = uuid::Uuid::parse_str(&str).map_err(|e| {
      napi::Error::new(
        napi::Status::GenericFailure,
        format!("Failed to parse UUID: {}", e),
      )
    })?;

    Ok(uuid.to_string())
  }

  /// Returns the string representation of the UUID.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.value.to_string()
  }
}
