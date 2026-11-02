placeholder_version := "0.0.0"

# Check formatting and lint both builder-only and derive-enabled configurations.
lint:
    cargo fmt --check
    cargo clippy --no-default-features --all-targets -- -D warnings
    cargo clippy --all-features --all-targets -- -D warnings

# Check compilation, integration tests, and doctests in both configurations.
check:
    cargo check --no-default-features --all-targets
    cargo check --all-features --all-targets
    cargo test --no-default-features --all-targets
    cargo test --no-default-features --doc
    cargo test --all-features --all-targets
    cargo test --all-features --doc

# Replace the package version temporarily (used for tagged releases).
set-version version:
    @sed -i.bak 's/^version = "{{placeholder_version}}".*/version = "{{version}}"/' Cargo.toml
    @rm Cargo.toml.bak
    cargo check

# Restore the source placeholder after local versioned builds.
reset-version version:
    @sed -i.bak 's/^version = "{{version}}".*/version = "{{placeholder_version}}"/' Cargo.toml
    @rm Cargo.toml.bak
    cargo check

# Print the next patch version derived from v* Git tags (requires doxxer).
next-version:
    @doxxer --tag-filter '^v' next patch

# Build the library with the next v*-tag-derived version, then restore 0.0.0.
build version=`doxxer --tag-filter '^v' next patch`:
    just set-version "{{version}}"
    cargo build
    just reset-version "{{version}}"
