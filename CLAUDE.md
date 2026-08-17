# Claude Code Configuration

## Project Overview

This is a pruned fork of Anki's Rust core library (`rslib`, published as the `anki`
crate), kept as a real fork (full git history, `upstream` remote pointing at
`ankitects/anki`) so changes can still be merged in selectively over time. Everything
outside the Rust core's build closure — the Python library, PyQt desktop UI, Svelte web
frontend, and the ninja-based monorepo build system — has been removed. See
[README.md](./README.md) for what's kept and why.

The intent is to consume this crate as a dependency (git or path) from another project,
not to run it as the Anki application. It targets both native and
`wasm32-unknown-unknown` (browser) callers. See [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
for what was removed/relocated and why, including the wasm32-specific adaptations.

## Layout

- `rslib/` — the `anki` crate itself: collection, cards, notes, notetypes, scheduler,
  search, import/export, sync, media, etc.
- `rslib/i18n`, `rslib/io`, `rslib/proto`, `rslib/proto_gen` — required build/runtime
  dependencies of `anki`.
- `proto/` — protobuf schema compiled by `rslib/proto`.
- `ftl/core`, `ftl/qt` (+ `core-repo`/`qt-repo` submodules) — Fluent translation strings;
  required at build time by `rslib/i18n`'s build script (it panics if missing).

## Building and testing

No `just`/ninja build system here — plain `cargo` directly, from the repo root:

```
cargo build -p anki
cargo test -p anki
cargo check -p anki
cargo clippy -p anki
(cd cargo/format && cargo fmt --manifest-path ../../rslib/Cargo.toml)
```

To check the wasm32 target (no test runner for it here, check/clippy only):

```
cargo check -p anki --target wasm32-unknown-unknown
```

(the required `--cfg getrandom_backend="wasm_js"` rustflag is baked into `.cargo/config.toml`)

Prerequisites: a `protoc` binary on `PATH` (or `PROTOC_BINARY` env var set), and the
`ftl/core-repo`/`ftl/qt-repo` submodules checked out (`git submodule update --init`).
`cargo fmt` needs the nightly toolchain pinned in `cargo/format/rust-toolchain.toml`
(hence running it from that directory with an explicit `--manifest-path`).

## Rust error handling

Use `error/mod.rs`'s `AnkiError`/`Result` and `snafu` within `rslib`. Unwrapping in
tests is fine.

## Merging from upstream

`upstream` remote points at `https://github.com/ankitects/anki.git`. Do not
`git merge upstream/main` directly into `main` — see
[docs/UPSTREAM_SYNC.md](./docs/UPSTREAM_SYNC.md) for the branch topology
(`main` stable / `next` staging for cherry-picks) and the
commit-by-commit cherry-pick workflow, including which upstream paths can be
skipped outright.
