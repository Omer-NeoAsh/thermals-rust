use std::num::NonZeroU64;

use super::Options;

fn parse(arguments: &[&str]) -> anyhow::Result<Options> {
    Options::parse(arguments.iter().map(|argument| argument.to_string()))
}

#[test]
fn no_arguments_means_run_until_stopped() {
    assert_eq!(parse(&[]).unwrap(), Options { samples: None });
}

#[test]
fn reads_sample_count() {
    assert_eq!(
        parse(&["--samples", "3"]).unwrap(),
        Options {
            samples: NonZeroU64::new(3)
        }
    );
}

#[test]
fn rejects_bad_arguments() {
    for arguments in [
        &["--samples"][..],
        &["--samples", "0"],
        &["--samples", "-1"],
        &["--samples", "three"],
        &["--verbose"],
    ] {
        assert!(
            parse(arguments).is_err(),
            "{arguments:?} should be rejected."
        );
    }
}
