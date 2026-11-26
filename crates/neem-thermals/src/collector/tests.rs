use std::num::NonZeroU64;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::{Collector, SAMPLE_INTERVAL, next_deadline};
use crate::fake_backend::FakeBackend;

#[test]
fn next_deadline_keeps_a_steady_beat() {
    let previous = Instant::now();
    let now = previous + Duration::from_secs(1);
    assert_eq!(next_deadline(previous, now), previous + SAMPLE_INTERVAL);
}

#[test]
fn next_deadline_does_not_catch_up_after_a_stall() {
    let previous = Instant::now();
    let now = previous + Duration::from_secs(60);
    assert_eq!(next_deadline(previous, now), now);
}

#[test]
fn stops_after_the_sample_limit() {
    let (_stop_sender, stop_receiver) = mpsc::channel();
    let collector = Collector {
        backend: Box::new(FakeBackend::new()),
        sample_limit: NonZeroU64::new(1),
        stop: stop_receiver,
    };

    let mut output = Vec::new();
    collector.run(&mut output).unwrap();

    assert_eq!(String::from_utf8(output).unwrap(), "fake/cpu: 45.0 °C\n");
}

#[test]
fn stops_when_asked_without_waiting_for_the_next_tick() {
    let (stop_sender, stop_receiver) = mpsc::channel();
    stop_sender.send(()).unwrap();
    let collector = Collector {
        backend: Box::new(FakeBackend::new()),
        sample_limit: None,
        stop: stop_receiver,
    };

    let started = Instant::now();
    let mut output = Vec::new();
    collector.run(&mut output).unwrap();

    assert_eq!(String::from_utf8(output).unwrap(), "fake/cpu: 45.0 °C\n");
    assert!(started.elapsed() < SAMPLE_INTERVAL);
}
