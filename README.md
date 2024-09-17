# Oxbuild

An ultra-fast `tsc`-like compiler built on top of [oxc](https://github.com/oxc-project/oxc).

## Features
- Transpiles TypeScript, JavaScript, JSX, and TSX
- Emits `.d.ts` files for TypeScript projects using `isolatedDeclarations`
- Source maps for transpiled code

## Installation

You can install `oxbuild` from [crates.io](https://crates.io/crates/oxbuild).
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

### TSConfigs

Oxbuild will respect `rootDir` and `outDir` settings in your `tsconfig.json`.
It will look for a `tsconfig.json` file next to the nearest `package.json` file
by default. If you want to specfiy a different `tsconfig.json` file, you can do

```sh
oxbuild --tsconfig path/to/tsconfig.json
```

