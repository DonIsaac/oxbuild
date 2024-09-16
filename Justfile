_default:
    @just --list -u

alias f := fix

# Apply formatting fixes
fmt:
    cargo fmt --all
    taplo fmt

# Apply all lint/format auto fixes. Stage all changes first.
fix:
    cargo clippy --no-deps --all-targets --all-features --fix --allow-staged
    just fmt
    git status
