use std::sync::Arc;

/// A cheaply clonable, zeroed on drop, String.
///
/// This is a simple newtype of `Arc<str>` that uses [`zeroize::Zeroize`] on last drop to avoid
/// keeping it around in memory.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct SecretString(Arc<str>);

impl SecretString {
    #[must_use]
    pub fn new(inner: Arc<str>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple(std::any::type_name::<Self>()).field(&"<secret>").finish()
    }
}

impl zeroize::Zeroize for SecretString {
    fn zeroize(&mut self) {
        if let Some(string) = Arc::get_mut(&mut self.0) {
            string.zeroize();
        }
    }
}

#[cfg(feature = "typesize")]
impl typesize::TypeSize for SecretString {
    fn extra_size(&self) -> usize {
        self.0.len() + (size_of::<usize>() * 2)
    }
}
