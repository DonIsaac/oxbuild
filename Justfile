_default:
    @just --list -u

alias f := fix
alias l := lint

version := `tq -f Cargo.toml 'package.version'`

# Install tools needed for development. Make sure cargo-binstall is installed first.
init:
    cargo binstall cargo-shear taplo-cli typos-cli -y

# Run Oxbuild (dev build, not optimized)
oxbuild *ARGS:
    cargo oxbuild {{ARGS}}

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
#
# requires these tools:
# - cargo-bump: https://github.com/wraithan/cargo-bump
# - tomlq:      https://github.com/cryptaliagy/tomlq
release semver_kind:
    # bail on uncommitted changes
    git diff --exit-code --name-only
    # replace package.version in Cargo.toml
    cargo bump {{semver_kind}}
    # update Cargo.lock
    cargo check
    @echo Creating release: {{version}}
    git add Cargo.toml Cargo.lock
    git commit -m "release: {{version}}"
    git tag v{{version}}
    git push --tags
    cargo publish

