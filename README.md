# Server

An all-in-one server ready for deployment.

```bash
# buidld in deverlopment mode
cargo run

# build a new release server
cargo build --release

# spin up a new deployment server
./target/release/server --host "localhost" --port 8080
./target/release/server --port 8080
```

## Modules

- route
- database
- authenticate
- rendering
- plugins
