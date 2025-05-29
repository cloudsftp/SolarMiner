use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug)]
pub struct DampenedSwitch {
    time_to_switch: Duration,
    command: bool,
    received_since: Instant,
}

impl DampenedSwitch {
    pub fn new(time_to_switch: Duration) -> Self {
        Self {
            time_to_switch,
            command: false,
            received_since: Instant::now(),
        }
    }

    pub fn perform(&mut self, command: bool) -> bool {
        if command != self.command {
            self.command = command;
            self.received_since = Instant::now();
            false
        } else {
            Instant::now().duration_since(self.received_since) > self.time_to_switch
        }
    }
}
