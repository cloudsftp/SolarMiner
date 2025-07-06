use anyhow::{Error, anyhow};
use std::{fmt::Debug, time::Duration};
use tokio::time::Instant;

#[derive(PartialEq, Clone)]
pub struct Part<T>
where
    T: Debug + PartialEq + Clone,
{
    value: Option<T>,
    last_update: Instant,
    default: T,
    timeout: Duration,
}

impl<T> Part<T>
where
    T: Debug + PartialEq + Clone,
{
    pub fn new(default: T, timeout: Duration) -> Self {
        Part {
            value: None,
            last_update: Instant::now(),
            default,
            timeout,
        }
    }

    pub fn set(&mut self, value: T) {
        self.value = Some(value);
        self.last_update = Instant::now();
    }

    fn outdated(&self) -> bool {
        Instant::now().duration_since(self.last_update) > self.timeout
    }

    fn initialized(&self) -> bool {
        self.value.is_some()
    }

    pub fn get_or_default(&self) -> T {
        (!self.outdated())
            .then_some(self.value.clone())
            .flatten()
            .unwrap_or(self.default.clone())
    }

    pub fn get_option(&self) -> Option<T> {
        if self.outdated() {
            return None;
        }

        self.value.clone()
    }

    // TODO: implement errors with thiserror?
    // - not initialized               -> ignore
    // - not initialized > timeout     -> error
    // - value outdated                -> error
    pub fn try_get(&self) -> Result<T, Error> {
        if self.outdated() {
            return Err(anyhow!("state outdated"));
        }

        if !self.initialized() {
            return Err(anyhow!("state not initialized"));
        }

        Ok(self.value.clone().unwrap())
    }
}

impl<T> Debug for Part<T>
where
    T: Debug + PartialEq + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.outdated() {
            f.write_fmt(format_args!(
                "outdated (since {:?})",
                Instant::now().duration_since(self.last_update),
            ))
        } else {
            match &self.value {
                None => f.write_str("not initialized"),
                Some(value) => Debug::fmt(&value, f),
            }
        }
    }
}
