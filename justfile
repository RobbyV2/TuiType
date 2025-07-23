# TuiType Project Task Runner
# Usage: just <command>
# Run `just --list` to see all available commands

# Default recipe - show available commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode  
build-release:
    cargo build --release

# Run the application
run *args:
    cargo run {{ args }}

# Run tests
test:
    cargo test

# Check the project (faster than build, just checks for errors)
check:
    cargo check

# Run clippy linter
clippy:
    cargo clippy

# Format code with rustfmt
fmt:
    cargo fmt

# Check if code is formatted correctly
fmt-check:
    cargo fmt -- --check

# Clean build artifacts
clean:
    cargo clean

# Auto-fix linting issues where possible
fix:
    cargo fix --allow-dirty --allow-staged
    cargo clippy --fix --allow-dirty --allow-staged

# Install the binary to ~/.cargo/bin
install:
    cargo install --path .

# Run all checks (useful for CI)
ci: fmt-check check clippy test

# Update dependencies
update:
    cargo update

# Show cargo tree of dependencies
deps:
    cargo tree

# Build for multiple platforms (using the existing build script)
build-multi:
    ./build_release.sh

# Show project info
info:
    @echo "TuiType - Terminal-based typing test application"
    @echo "Version: $(grep '^version =' Cargo.toml | cut -d '"' -f 2)"
    @echo "Build targets available via build-multi:"
    @echo "  - Linux x86_64"
    @echo "  - macOS x86_64" 
    @echo "  - macOS ARM (Apple Silicon)"
    @echo "  - Windows x86_64"
    @echo "  - WebAssembly (WASI)"
    @echo "  - WebAssembly (Web)"

# Development workflow - format, check, test
dev: fmt check test

# Release workflow - all checks plus release build
release: ci build-release

# Quick check - just build and clippy (fastest feedback)
quick: check clippy