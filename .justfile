# Default recipe to display available commands
default:
    @just --list

# Build the library
build:
    cargo build

# Build the library in release mode
build-release:
    cargo build --release

# Build all workspace members
build-all:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run clippy linter
lint:
    cargo clippy --workspace -- -D warnings

# Check code formatting
fmt-check:
    cargo fmt --all --check

# Format code
fmt:
    cargo fmt --all

# Generate OpenAPI schema from hello_world example
schema:
    cargo run -p hello_world -- --test-schema

# Generate OpenAPI schema with pretty JSON formatting (requires jq)
schema-pretty:
    cargo run -p hello_world -- --test-schema | jq '.'

# View all schemas in the OpenAPI spec (requires jq)
schema-components:
    cargo run -p hello_world -- --test-schema | jq '.components.schemas'

# View specific endpoint in the OpenAPI spec (requires jq)
schema-endpoint endpoint:
    cargo run -p hello_world -- --test-schema | jq '.paths."{{endpoint}}"'

# Run the hello_world example server
serve:
    cargo run -p hello_world

# Run the hello_world example server in release mode
serve-release:
    cargo run -p hello_world --release

# Clean build artifacts
clean:
    cargo clean

# Check for unused dependencies
check-deps:
    cargo check --workspace

# Run all quality checks (fmt, clippy, test)
check: fmt-check lint test

# Build documentation
docs:
    cargo doc --workspace --no-deps

# Build and open documentation in browser
docs-open:
    cargo doc --workspace --no-deps --open