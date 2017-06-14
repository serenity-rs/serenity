use chrono::{DateTime, Duration, UTC};
use std::thread;
use std::time::Duration as StdDuration;

pub struct Timer {
    due: DateTime<UTC>,
    duration: Duration,
}

impl Timer {
    pub fn new(duration_in_ms: u64) -> Timer {
        let duration = Duration::milliseconds(duration_in_ms as i64);

        Timer {
            due: UTC::now() + duration,
            duration: duration,
        }
    }

    pub fn await(&mut self) {
        let due_time = (self.due.timestamp() * 1000) + self.due.timestamp_subsec_millis() as i64;
        let now_time = {
            let now = UTC::now();

            (now.timestamp() * 1000) + now.timestamp_subsec_millis() as i64
        };

        if due_time > now_time {
            let sleep_time = due_time - now_time;

            if sleep_time > 0 {
                thread::sleep(StdDuration::from_millis(sleep_time as u64));
            }
        }

        self.due = self.due + self.duration;
    }

    pub fn check(&mut self) -> bool {
        if UTC::now() >= self.due {
            self.due = self.due + self.duration;

            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.due = UTC::now() + self.duration;
    }
}
