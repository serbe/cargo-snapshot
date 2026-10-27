# cargo-snapshot

`cargo-snapshot` creates a Markdown snapshot of the current Cargo project,
including its dependencies, project tree, and Rust source files reachable from
the root package's binary and library targets.

## Usage

Run from a Cargo project directory:

```sh
cargo snapshot
```

Or invoke the binary directly:

```sh
cargo-snapshot --output snapshot.md
```

Options:

- `-o, --output <PATH>`: choose the output path (default: `snapshot.md`).
- `-l, --log-level <LEVEL>`: set the log level (`error`, `warn`, `info`,
  `debug`, or `trace`).
- `--readme`: include the package README when available.
- `--no-comments`: remove comments from Rust source files in the snapshot.
