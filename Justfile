# Lint the code
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Build the project (Release mode)
build:
    cargo build --release

# Run tests
test:
    cargo test --all-features --verbose

# Build for Windows using cross
windows-build:
    cross build --target=x86_64-pc-windows-gnu --release

# Run tests for Windows using cross
windows-test:
    cross test --target=x86_64-pc-windows-gnu --verbose
