use chrono::{DateTime, Duration, Utc};
use std::{
    time::Duration as StdDuration,
    thread
};

#[derive(Debug)]
pub struct Timer {
    due: DateTime<Utc>,
    duration: Duration,
}

impl Timer {
    pub fn new(duration_in_ms: u64) -> Timer {
        let duration = Duration::milliseconds(duration_in_ms as i64);

        Timer {
            due: Utc::now() + duration,
            duration,
        }
    }

    pub fn r#await(&mut self) {
        let due_time = (self.due.timestamp() * 1000) + i64::from(self.due.timestamp_subsec_millis());
        let now_time = {
            let now = Utc::now();

            (now.timestamp() * 1000) + i64::from(now.timestamp_subsec_millis())
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
        if Utc::now() >= self.due {
            self.due = self.due + self.duration;

            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) { self.due = Utc::now() + self.duration; }
}
