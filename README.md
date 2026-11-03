# cargo-snapshot

`cargo-snapshot` creates a Markdown snapshot of the current Cargo project,
including its dependencies, project tree, and Rust source files reachable from
the root package's binary and library targets.

## Install

Install the published package with Cargo:

```sh
cargo install cargo-snapshot
```

To install from a local checkout instead:

```sh
cargo install --path .
```

Cargo installs the `cargo-snapshot` executable in its binary directory
(usually `~/.cargo/bin` on Unix-like systems or `%USERPROFILE%\\.cargo\\bin`
on Windows). Make sure that directory is on `PATH`.

## Usage

Run from a Cargo project directory:

```sh
cargo snapshot
```

Pass options after the subcommand name:

```sh
cargo snapshot --output snapshot.md --readme
```

The installed executable can also be invoked directly:

```sh
cargo-snapshot --output snapshot.md
```

Check that Cargo can find the subcommand with:

```sh
cargo snapshot --help
```

Options:

- `-o, --output <PATH>`: choose the output path (default: `snapshot.md`).
- `-l, --log-level <LEVEL>`: set the log level (`error`, `warn`, `info`,
  `debug`, or `trace`).
- `--readme`: include the package README when available.
- `--no-comments`: remove comments from Rust source files in the snapshot.
- `--features <FEATURES>`: enable Cargo features when resolving `#[cfg]` modules.
- `--all-features`: resolve modules with every package feature enabled.
- `--no-default-features`: resolve modules without default features.
- `--target <TRIPLE>`: evaluate target-specific `#[cfg]` attributes for a target triple.
