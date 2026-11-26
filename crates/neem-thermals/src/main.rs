mod collector;
#[cfg(any(test, feature = "fake"))]
mod fake_backend;
mod options;

use std::io;
use std::process::ExitCode;
use std::sync::mpsc;

use anyhow::Context;
use neem_thermals_core::Backend;

use crate::collector::Collector;
use crate::options::Options;

const STARTUP_FAILURE: u8 = 78;

fn main() -> ExitCode {
    match start() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(STARTUP_FAILURE)
        }
    }
}

fn start() -> anyhow::Result<()> {
    let options = Options::parse(std::env::args().skip(1))?;

    let Some(backend) = platform_backend() else {
        eprintln!("warning: no sensor backend for this platform yet.");
        return Ok(());
    };

    let (stop_sender, stop_receiver) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = stop_sender.send(());
    })
    .context("installing the Ctrl-C handler")?;

    let collector = Collector {
        backend,
        sample_limit: options.samples,
        stop: stop_receiver,
    };
    collector.run(&mut io::stdout())
}

#[cfg(feature = "fake")]
fn platform_backend() -> Option<Box<dyn Backend>> {
    Some(Box::new(fake_backend::FakeBackend::new()))
}

#[cfg(not(feature = "fake"))]
fn platform_backend() -> Option<Box<dyn Backend>> {
    None
}
