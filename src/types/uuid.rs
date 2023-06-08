use napi::Result as NapiResult;

/// `Uuid` and `Timeuuid` are represented through this type.
#[derive(Debug)]
#[napi]
pub struct Uuid {
  pub(crate) value: uuid::Uuid,
}

impl Uuid {
  /// Returns the underlying `uuid::Uuid` value.
  pub fn value(&self) -> uuid::Uuid {
    self.value
  }
}

impl From<uuid::Uuid> for Uuid {
  fn from(value: uuid::Uuid) -> Self {
    Self { value }
  }
}

impl From<Uuid> for uuid::Uuid {
  fn from(value: Uuid) -> Self {
    value.value
  }
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
  pub fn from_string(str: String) -> NapiResult<Uuid> {
    let uuid = uuid::Uuid::parse_str(&str).map_err(|e| {
      napi::Error::new(
        napi::Status::GenericFailure,
        format!("Failed to parse UUID: {}", e),
      )
    })?;

    Ok(Self { value: uuid })
  }

  /// Returns the string representation of the UUID.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.value.to_string()
  }
}
