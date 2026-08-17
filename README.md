# anki-rslib-core

A pruned fork of [Anki](https://apps.ankiweb.net)'s Rust core (`rslib`, the `anki`
crate), meant to be consumed as a Cargo dependency (git or path) rather than run as
the full Anki application. This repo intentionally keeps only what's needed to build
the `anki` crate, plus its bundled multi-language translation strings — the Python
library, PyQt desktop UI, Svelte web frontend, and the ninja-based monorepo build
system have all been removed.

This is a real fork with intact git history (not a history-rewritten extraction), so
upstream changes from [ankitects/anki](https://github.com/ankitects/anki) can still be
merged in selectively over time — see [docs/UPSTREAM_SYNC.md](./docs/UPSTREAM_SYNC.md)
for that workflow, and [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) for what was
removed/relocated and why (including `wasm32-unknown-unknown` support, which this fork
builds cleanly for, in addition to native).

## What's here

- `rslib/` — the `anki` crate: collections, cards, notes, notetypes, the scheduler,
  search, import/export, sync, media handling, etc.
- `rslib/i18n`, `rslib/io`, `rslib/proto`, `rslib/proto_gen` — its required build/
  runtime dependencies (translation loading, I/O error helpers, protobuf codegen).
- `proto/` — the protobuf schema `rslib/proto` compiles.
- `ftl/core` + `ftl/qt` (and their `core-repo`/`qt-repo` submodules) — Anki's Fluent
  translation strings, in every language Anki ships. Both are required at build time
  by `rslib/i18n`'s build script.

## Building

Requires:

- The Rust toolchain pinned in `rust-toolchain.toml`.
- A `protoc` (Protocol Buffers compiler) binary on `PATH`, or set the `PROTOC_BINARY`
  env var to point at one.
- The `ftl/core-repo` and `ftl/qt-repo` git submodules checked out:
  `git submodule update --init`

Then, from the repo root:

```
cargo build -p anki
cargo test -p anki
```

There is no `just`/ninja build system in this fork — plain `cargo` commands are used
directly. To check the `wasm32-unknown-unknown` (browser) target:

```
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo check -p anki --target wasm32-unknown-unknown
```

## License

Anki, and this fork, are licensed under the GNU AGPL v3 or later — see [LICENSE](./LICENSE).
Contributors are listed in [CONTRIBUTORS](./CONTRIBUTORS).
