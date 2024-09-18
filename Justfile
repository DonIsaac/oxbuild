_default:
    @just --list -u

alias f := fix
alias l := lint

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
