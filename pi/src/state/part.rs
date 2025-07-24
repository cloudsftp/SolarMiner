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
    control_timeout: Duration,
}

impl<T> Part<T>
where
    T: Debug + PartialEq + Clone,
{
    pub fn new(default: T, timeout: Duration, control_timeout: Duration) -> Self {
        Part {
            value: None,
            last_update: Instant::now(),
            default,
            timeout,
            control_timeout,
        }
    }

    pub fn set(&mut self, value: T) {
        self.value = Some(value);
        self.last_update = Instant::now();
    }

    fn outdated(&self) -> bool {
        Instant::now().duration_since(self.last_update) > self.timeout
    }

    pub fn get_option(&self) -> Option<T> {
        (!self.outdated()).then(|| self.value.clone()).flatten()
    }

    fn control_outdated(&self) -> bool {
        Instant::now().duration_since(self.last_update) > self.control_timeout
    }

    pub fn get_control_or_default(&self) -> T {
        self.get_control_option().unwrap_or(self.default.clone())
    }

    pub fn get_control_option(&self) -> Option<T> {
        (!self.control_outdated())
            .then(|| self.value.clone())
            .flatten()
    }
}

impl<T> Debug for Part<T>
where
    T: Debug + PartialEq + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.control_outdated() {
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
