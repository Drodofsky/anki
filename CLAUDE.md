# Claude Code Configuration

## Project Overview

This is a pruned fork of Anki's Rust core library (`rslib`, published as the `anki`
crate), kept as a real fork (full git history, `upstream` remote pointing at
`ankitects/anki`) so changes can still be merged in selectively over time. Everything
outside the Rust core's build closure — the Python library, PyQt desktop UI, Svelte web
frontend, and the ninja-based monorepo build system — has been removed. See
[README.md](./README.md) for what's kept and why.

The intent is to consume this crate as a dependency (git or path) from another project,
not to run it as the Anki application.

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
cargo fmt
```

Prerequisites: a `protoc` binary on `PATH` (or `PROTOC_BINARY` env var set), and the
`ftl/core-repo`/`ftl/qt-repo` submodules checked out (`git submodule update --init`).

## Rust error handling

Use `error/mod.rs`'s `AnkiError`/`Result` and `snafu` within `rslib`. Unwrapping in
tests is fine.

## Merging from upstream

`upstream` remote points at `https://github.com/ankitects/anki.git`. Since large parts
of the original repo have been deleted here, a straight `git merge upstream/main` will
raise modify/delete conflicts for every removed path — resolve those by keeping the
deletion (`git rm <path>`) unless the change is specifically worth pulling in. Prefer
targeted `git checkout upstream/main -- <path>` for individual files/directories over a
full merge when only chasing a specific upstream fix.
