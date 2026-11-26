use std::num::NonZeroU64;

use anyhow::{Context, bail};

#[derive(Debug, PartialEq)]
pub struct Options {
    pub samples: Option<NonZeroU64>,
}

impl Options {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> anyhow::Result<Self> {
        let mut options = Options { samples: None };
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--samples" => {
                    let count = arguments.next().context("--samples needs a number")?;
                    let samples = count.parse().with_context(|| {
                        format!("--samples needs a whole number above 0, got {count:?}")
                    })?;
                    options.samples = Some(samples);
                }
                unknown => bail!("unknown argument {unknown:?}"),
            }
        }

        Ok(options)
    }
}

#[cfg(test)]
mod tests;
