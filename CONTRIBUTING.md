# Contributing

Thanks for contributing to Oxbuild!

The best ways to get involved are:
1. Look through [good first issues](https://github.com/DonIsaac/oxbuild/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) and pick something that looks interesting
2. Get involved with [oxc](https://github.com/oxc-project/oxc), which powers Oxbuild.

## Setup

After you've cloned this repository, here's what you need to get set up:

1. Make sure you've installed Rust. If you haven't yet, you can install it with [`rustup`](https://www.rust-lang.org/tools/install).
2. Install [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file#installation), a tool for downloading cargo binaries without needing to compile them first.
   ```sh
   # Click the link above for more ways to install this
   cargo install cargo-binstall
   ```
2. Install [`just`](https://github.com/casey/just?tab=readme-ov-file#installation), a `make`-like task runner.
   ```sh
   # Click the link above for more ways to install just
   cargo binstall just
   ```
3. Install other binaries for linting/checking:
   ```sh
   just init
   ```

Look through `Justfile` to see all available scripts, or just run `just` (with no arguments) to see a help message of all commands available.

## Running
You can use `cargo oxbuild` as a `cargo run` shorthand. I personally find it easier than typing multiple CLI args each time :)

## Testing/Linting

Our CI jobs will catch `rustfmt` and `clippy` issues automatically. You can run `just fix` to apply all possible fixes automatically; just make sure you've staged your changes first!

```sh
git co -b you/feat/cool-feature
# ... make changes ...
git add -A
just fix
# review what fixes were applied, then stage them when you're ready
g add -A
# lint again to make sure everything's ok
just lint
# good to go!
git push -u origin you/feat/cool-feature
```
