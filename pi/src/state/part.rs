use anyhow::{Error, anyhow};
use std::{fmt::Debug, time::Duration};
use tokio::time::Instant;

#[derive(Debug, PartialEq, Clone)]
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
        self.last_update.duration_since(self.last_update) > self.timeout
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
        todo!()
    }
}
