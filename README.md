# thermals-rust
Computer thermals / thermal throttling
## Commands
### Running
You can run a specific crate or the entire application. We also have runs that are feature specific. 

`cargo run -p neem-thermals --all-features` OR `cargo run -p neem-thermals --features fake -- --samples 3`

`cargo run -p neem-thermals`  
### Testing
`cargo test --workspace --all-features`

`cargo test --workspace`
### Formatting / Checking
`cargo fmt --all`

`cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Features
We currently have a fake-backend setup under the `fake` feature. This will allow you to test sensor information to 
ensure that the collector is properly running its loop. 

When running for real, ensure that all features are off and not selected to be run.