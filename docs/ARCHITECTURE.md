# Architecture of this fork

This document explains how this repo diverges from upstream
[ankitects/anki](https://github.com/ankitects/anki), and why - useful both for
orienting yourself in the pruned tree, and for judging whether a given upstream
commit is relevant when cherry-picking (see [UPSTREAM_SYNC.md](./UPSTREAM_SYNC.md)).

## Goal

Consume `rslib` (the `anki` crate) as a plain Rust library from native and
`wasm32-unknown-unknown` (browser) callers, calling async functions directly
and awaiting them - not through any byte-marshalled FFI boundary. Everything
kept exists to serve that; everything removed existed only to serve some other
consumption model (Python FFI, a bundled Qt/web desktop app, a self-hosted sync
server, JNI for Android).

## Guiding principles for changes in this repo

Both of these exist to keep future upstream cherry-picks (see
[UPSTREAM_SYNC.md](./UPSTREAM_SYNC.md)) tractable indefinitely, not just at the
time of the initial prune:

- **Minimal diff from upstream.** Prefer the smallest change that achieves the
  goal - additive/`#[cfg]`-gated over restructuring, keep upstream's file
  layout and naming where the code itself is unchanged. Every wasm32
  adaptation in this repo follows this: native code paths are untouched, wasm
  gets a parallel `#[cfg(target_arch = "wasm32")]` arm alongside it (see
  `io_monitor.rs`, `full_sync.rs`, `editor.rs`) rather than a rewrite of the
  shared logic. A smaller diff from `upstream/main` is a smaller/cleaner
  conflict surface on every future cherry-pick, forever.
- **Only core Anki functionality lives here.** This crate stays a (pruned,
  wasm-enabled) generic Anki core - not a place for app-specific business
  logic. A downstream consumer's own domain modeling, UI state, or
  vocab-app-specific features belong in that consumer's own crate, which
  depends on this one. If something feels like "my app needs this," it
  probably doesn't belong in this repo; if it feels like "any Anki-core
  consumer would want this," it does. Keeping that boundary sharp is what
  keeps this repo mergeable with upstream instead of drifting into being
  something else's fork-of-a-fork.

## What's kept vs. removed (top level)

See [README.md](../README.md) for the file-level list. In short: `rslib/` and
its required build deps (`rslib/{i18n,io,proto,proto_gen}`), `proto/`,
`ftl/{core,qt}` (+ submodules). Everything Python/Qt/Svelte/ninja-build is gone.

## Removed: the self-hosted sync server

`rslib/src/sync/http_server/` (routes, handlers, the server-side media
database) is gone entirely. This fork is sync-*client*-only - it talks to a
remote sync server (AnkiWeb or someone else's self-hosted instance), never
runs one itself.

This wasn't just a size trim: every domain module (`sync/collection/*`,
`sync/media/*`, `sync/request/`, `sync/response.rs`) had server-side handler
functions and axum extractor impls threaded through it alongside the client
logic, using axum's `Request`/`Response`/`Multipart` types as shared
vocabulary. Removing the server side meant auditing each file to find what was
genuinely server-only (mostly free functions prefixed `server_*`, plus
`axum::extract::FromRequest`/`IntoResponse` impls) versus shared/client logic,
then stripping just the former.

**Practical effect for cherry-picking:** any upstream commit touching
`sync/http_server/`, `sync/media/database/server/`, or a `server_*` function
in `sync/collection/*.rs` can be skipped outright.

## Removed: `Backend`, the synchronous FFI wrapper

`rslib/src/backend/` (the `Backend` struct and its `impl Backend*Service`
blocks) is gone. `Backend` existed to bridge async operations into synchronous
methods - via a background multi-thread tokio runtime and `.block_on()` - for
byte-marshalled callers: the (already-removed) Python bridge `pylib/rsbridge`,
and AnkiDroid's JNI bridge. Its actual dispatch entrypoints
(`run_db_command_bytes`, `init_backend`) had zero callers left in this fork
once those consumers were gone.

Every protobuf service in Anki's `.proto` files has two flavors: a plain
`{Name}Service` (implemented directly on `Collection`, real business logic)
and a `Backend{Name}Service` (implemented on `Backend`, usually just
delegating to the `Collection` version via `Backend::with_col(...)`). See
`rslib/rust_interface.rs` and `rslib/proto_gen/src/lib.rs::get_services()` for
how this split is generated. Since native/wasm callers now call `Collection`'s
plain trait methods directly, the delegating `Backend` layer added nothing.

`rust_interface.rs` (the codegen behind `rslib/src/services.rs`) no longer
generates `Backend*Service` traits or any byte-in/byte-out dispatch code at
all - only the plain `{Name}Service` traits on `Collection`.

**Where backend/ logic actually went** (nothing was silently dropped without
review - see the commit "Remove Backend, the synchronous FFI wrapper" for the
full reasoning per file):

| Was in `backend/...`             | Now in                                          | Why |
|-----------------------------------|--------------------------------------------------|-----|
| `config.rs`                       | `config/service.rs`                              | Misfiled - implemented `ConfigService` on `Collection`, no `Backend` trait at all |
| `ops.rs`                          | `ops.rs` (root)                                  | Misfiled - `OpChanges`/`UndoStatus` proto conversions, used in 34+ files crate-wide |
| `dbproxy.rs`                      | `ankidroid/dbproxy.rs`                           | Backs the real `AnkidroidService`, actively used |
| `ankihub.rs`, `ankiweb.rs`        | `ankihub/login.rs`, `ankiweb.rs` (new)           | Was already just `.block_on()`-wrapping existing async fns; now called directly |
| `media.rs`'s `add_media_from_url` | `media/service.rs`, re-exported from `media/mod.rs` | Turned into a plain `pub async fn` |
| `sync.rs`                         | `sync/collection/{status,normal}.rs`, `sync/login.rs`, `sync/http_client/mod.rs::build_client` | Proto conversions and the 300s remote-status cache kept (caller-owned now); OS-thread+`AbortHandle` background-sync machinery dropped - a caller can `tokio::spawn` the existing async methods directly and use the returned `JoinHandle`'s own `.abort()` |
| `scheduler/service/mod.rs`'s `BackendSchedulerService` impl | `scheduler/service/mod.rs` (plain fns), re-exported from `scheduler/mod.rs` | FSRS param computation/benchmark didn't touch `Backend` state at all |
| `i18n.rs`, `card_rendering.rs`'s `strip_html`, `collection.rs`'s progress/abort methods | *(deleted)* | Pure duplicates of the `Collection`-side service methods |
| `error.rs` (`AnkiError::into_protobuf`), `github.rs`, `adding.rs`'s legacy timing/debug shims | *(deleted)* | Dead - their only caller was the removed FFI dispatch |

**Practical effect for cherry-picking:** upstream commits to `backend/*.rs` need
per-commit judgment - check the table above for where the equivalent code lives
now, or whether it was dropped as dead/duplicate. Commits to `backend/mod.rs`
itself (the `Backend` struct, `runtime_handle`, `with_col`) can be skipped -
there's no equivalent anymore.

Also removed as part of the same pass: the desktop self-updater
(`updates.rs`, `backend/github.rs`'s real implementation, now a stub returning
`invalid_input!` since the generated dispatch still requires *something*
implement `BackendGithubService`) - it downloads new Anki *installer*
releases, meaningless outside the native desktop app.

## wasm32-unknown-unknown support

`cargo check -p anki --target wasm32-unknown-unknown` is clean (the required
`--cfg getrandom_backend="wasm_js"` rustflag lives in `.cargo/config.toml`, so
no manual env var is needed). Native behavior is unchanged throughout - every
wasm-specific adaptation is behind
`#[cfg(target_arch = "wasm32")]` or a target-specific Cargo dependency, and the
native path was re-verified (`cargo build`/`test`/`clippy`/`fmt`) after each one.

- **`rusqlite` 0.36 -> 0.40.2**: 0.40 uses `sqlite-wasm-rs` (a prebuilt WASM
  SQLite build) on `wasm32-unknown-unknown` automatically, instead of
  compiling the C amalgamation (impossible - that target has no libc at all).
  Fallout: rusqlite 0.40 dropped blanket `u64`/`usize` `ToSql`/`FromSql` impls;
  9 call sites now cast through `i64` at the DB boundary.
- **`zstd`'s `zstdmt` feature**: needs pthreads, native-only now
  (`rslib/Cargo.toml`'s `[target.'cfg(...)'.dependencies]` split). One
  `.multithread()` call site in `import_export/package/colpkg/export.rs` is
  `#[cfg]`-gated to match.
- **`tokio`**: dropped `rt-multi-thread`/`fs`/`signal` (all now-unused after
  removing `Backend`/the self-updater/the sync server); wasm32 gets `sync`+
  `time` added on top via a target-specific dependency entry (tokio's own
  compile-time check only allows `sync,macros,io-util,rt,time` on that target).
- **`getrandom`**: needs its `wasm_js` feature explicitly enabled (both major
  versions 0.3 and 0.4, pulled in transitively at different versions by `rand`
  via `anki_io` and via `fsrs` respectively), plus the `--cfg
  getrandom_backend="wasm_js"` RUSTFLAG.
- **`sync/http_client/io_monitor.rs`**: split into `native`/`wasm` submodules
  behind the same `IoMonitor::new()`/`zstd_request_with_timeout()` API.
  reqwest's wasm `Body` can only be constructed from a full `Vec<u8>` (its
  `wrap_stream` needs the native-only `stream` feature) - so the wasm path
  buffers the zstd-compressed body instead of streaming it, and skips
  per-transfer progress reporting entirely (browsers' `fetch()` has no
  upload-progress mechanism in any browser, a real platform gap - getting one
  would mean bypassing `fetch`/reqwest for `web_sys::XmlHttpRequest`'s
  `upload.onprogress`, not attempted here). It uses reqwest's own per-request
  `.timeout()` in place of the native stall-detector. `sync/http_client/
  full_sync.rs` has the matching native/wasm split for the same reason.
- **`async_trait`**: defaults to requiring `Send` futures;
  `wasm-bindgen-futures`' `JsFuture` is `!Send` (JS values can't cross
  threads). `SyncProtocol`/`MediaSyncProtocol` and their `HttpSyncClient` impls
  use `#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]` - native keeps
  the `Send` bound (so a native caller can still `tokio::spawn` a sync
  operation onto a multi-threaded runtime).
- **`editor.rs`**: local `file://` URL handling (`std::fs::read`) is
  native-only - browsers block `fetch()` from reading `file://` URLs entirely,
  and there's no filesystem to read from regardless.
  `reqwest::ClientBuilder::timeout()` doesn't exist on wasm32; moved to
  `RequestBuilder::timeout()` (works identically on both targets).

**Practical effect for cherry-picking:** upstream commits touching
`sync/http_client/{io_monitor,full_sync}.rs`, `sync/collection/protocol.rs`,
`sync/media/protocol.rs`, or `editor.rs` need care to preserve the wasm32 split
- re-apply the equivalent change to both the native and wasm arms, not just
whichever one the upstream diff happens to touch.
