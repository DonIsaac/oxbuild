_default:
    @just --list -u

alias f := fix
alias l := lint

# Install tools needed for development. Make sure cargo-binstall is installed first.
init:
    cargo binstall cargo-shear taplo-cli typos-cli -y

# Run Oxbuild (dev build, not optimized)
oxbuild *ARGS:
    cargo oxbuild {{ARGS}}

# Alias for `cargo build`
build:
    cargo build

# Alias for `cargo test`
test:
    cargo test


# Apply formatting fixes
fmt:
    @cargo fmt --all
    @taplo fmt

lint:
    cargo clippy --no-deps --all-targets --all-features -- -D warnings

# Apply all lint/format auto fixes. Stage all changes first.
fix:
    cargo clippy --no-deps --all-targets --all-features --fix --allow-staged
    just fmt
    git status

# Make a release. `semver_kind` is major/minor/patch
release semver_kind:
    # bail on uncommitted changes
    git diff --exit-code --name-only
    cargo ck
    cargo release {{semver_kind}} --execute
