# Oxbuild

[![CI](https://github.com/DonIsaac/oxbuild/actions/workflows/ci.yml/badge.svg)](https://github.com/DonIsaac/oxbuild/actions/workflows/ci.yml)
[![Crates.io Version](https://img.shields.io/crates/v/oxbuild)](https://crates.io/crates/oxbuild)
[![NPM Version](https://img.shields.io/npm/v/oxbuild)](https://npmjs.com/package/oxbuild)
[![License](https://img.shields.io/crates/l/oxbuild)](./LICENSE)

An ultra-fast `tsc`-like compiler built on top of [oxc](https://github.com/oxc-project/oxc).

> #### 🚧 Under Construction
>
> Both Oxbuild and oxc are actively under construction and are not yet suitable for production use. If you find a bug in either project, we would love for you to open an issue on GitHub!

## Features

- Transpile TypeScript, JavaScript, JSX, and TSX
- Emit `.d.ts` files for TypeScript projects that use
  [`isolatedDeclarations`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html#isolated-declarations)
- JS Source maps for transpiled code

## Installation

You can install `oxbuild` from [npm](https://www.npmjs.com/package/oxbuild).

```sh
npm install -g oxbuild
```

Or from [crates.io](https://crates.io/crates/oxbuild).

```sh
# using cargo-binstall (recommended)
cargo binstall oxbuild
# or using cargo
cargo install oxbuild
```

Or, build it from source

```sh
git clone git@github.com:DonIsaac/oxbuild.git
cd oxbuild
cargo build --release --bin oxbuild
cp target/release/oxbuild /usr/local/bin
```

## Usage

Assuming you are in your project's root directory and your source code is all in
`./src`, you can compile your project to `./dist` by running:

```sh
oxbuild
```

If `oxbuild` is behaving in an unexpected way, please run it with debug logs and
create a new issue on GitHub.

```sh
RUST_LOG=debug oxbuild
```

### TSConfig Support

Oxbuild will respect `rootDir` and `outDir` settings in your `tsconfig.json`.
It will look for a `tsconfig.json` file next to the nearest `package.json` file
by default. If you want to specfiy a different `tsconfig.json` file, you can do

```sh
oxbuild --tsconfig path/to/tsconfig.json
```

### TypeScript Declarations

To generate `.d.ts` files, your project must have
[`isolatedDeclarations`](https://www.typescriptlang.org/tsconfig/#isolatedDeclarations)
enabled. After that, `.d.ts` files will be automatically emitted on each build.
