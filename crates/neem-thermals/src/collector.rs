use std::io::Write;
use std::num::NonZeroU64;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use anyhow::Context;
use neem_thermals_core::Backend;

pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(5);

pub struct Collector {
    pub backend: Box<dyn Backend>,
    pub sample_limit: Option<NonZeroU64>,
    pub stop: Receiver<()>,
}

impl Collector {
    pub fn run(mut self, output: &mut dyn Write) -> anyhow::Result<()> {
        let sensors = self.backend.discover().context("discovering sensors")?;
        if sensors.is_empty() {
            eprintln!("warning: no sensors found.");
            return Ok(());
        }

        let mut ticks_taken: u64 = 0;
        let mut deadline = Instant::now();

        loop {
            match self.backend.sample() {
                Ok(samples) => {
                    for sample in samples {
                        writeln!(output, "{sample}")?;
                    }
                }
                Err(error) => eprintln!("warning: skipped a tick: {error}"),
            }

            ticks_taken += 1;
            if self
                .sample_limit
                .is_some_and(|limit| ticks_taken >= limit.get())
            {
                return Ok(());
            }

            deadline = next_deadline(deadline, Instant::now());
            let wait = deadline.saturating_duration_since(Instant::now());
            match self.stop.recv_timeout(wait) {
                Err(RecvTimeoutError::Timeout) => {}
                Ok(()) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }
    }
}

fn next_deadline(previous: Instant, now: Instant) -> Instant {
    (previous + SAMPLE_INTERVAL).max(now)
}

#[cfg(test)]
mod tests;
