# Syncing from upstream

This fork tracks [ankitects/anki](https://github.com/ankitects/anki) as the
`upstream` remote, but is pruned too heavily (see
[ARCHITECTURE.md](./ARCHITECTURE.md)) for a plain `git merge upstream/main` to
be practical - it would turn every file we deleted into a modify/delete
conflict, forever, on every sync. Instead: review upstream's commits, skip the
ones that don't apply, and cherry-pick the rest individually.

## Last reviewed upstream commit

```
bfaf62d4dd0224607e2dd453ae6775b5bc2e6833  (2026-08-18)
```

Everything up to and including this commit has been checked against `main`
(either cherry-picked, or explicitly skipped as irrelevant). **Update this
line whenever you finish a review pass**, even if nothing ended up being
cherry-picked - it marks how far the review has gotten, not how much was
taken.

## Branches

- **`main`** - the real, stable branch. What you'd build against. Only
  receives commits that have been reviewed and verified. There is no local
  mirror branch of upstream; diff/review directly against the `upstream`
  remote (`upstream/main`) instead - `git fetch upstream` keeps that
  remote-tracking ref current without needing a local branch of its own.
- **`next`** - disposable staging branch for testing a batch of cherry-picks
  before they touch `main`. Recreated fresh each sync round (see below) -
  don't build anything on top of it that you want to keep.

Using a staging branch here is worth the small overhead: cherry-picking across
a fork this divergent is exactly the kind of thing that goes fine for the
first few commits in a batch and then hits a conflict or a subtle breakage
three commits later. Verifying the *whole batch* together on `next` before
`main` ever moves means `main` stays in a known-good state the entire time -
if something in the batch turns out broken, you fix it or drop that one
commit on `next` and re-verify, instead of having to figure out how to unwind
a bad cherry-pick that's already landed on your real branch.

## Workflow

1. **Fetch upstream.**
   ```
   git fetch upstream
   ```

2. **See what's new.**
   ```
   git log --oneline <last-reviewed-sha>..upstream/main
   ```
   For each commit, `git show --stat <sha>` first. Skip immediately (no
   further review needed) if it *only* touches paths that don't exist in this
   fork: `qt/`, `ts/`, `pylib/`, `docs/`, `build/`, `tools/`,
   `rslib/src/sync/http_server/`, `rslib/src/sync/media/database/server/`, a
   `server_*` function in `rslib/src/sync/collection/*.rs`, or
   `rslib/src/backend/mod.rs` (see ARCHITECTURE.md's tables for the full
   removed/relocated lists). This is the overwhelming majority of commits.

   For anything touching surviving code (`rslib/src/{cards,notes,decks,
   notetype,scheduler,search,storage,import_export,...}`, shared `sync/`
   client code, or a file ARCHITECTURE.md's table says was relocated),
   actually read the diff.

3. **Stage the batch on `next`.**
   ```
   git checkout main
   git branch -f next        # reset next to the current tip of main
   git checkout next
   git cherry-pick <sha1> <sha2> ...
   ```
   For a commit that's relevant but also touches deleted files, use
   `git cherry-pick -n <sha>` (stages without committing) so you can
   `git restore --staged --worktree <deleted-path>` the irrelevant hunks
   before committing.

4. **Verify on `next`.**
   ```
   cargo build -p anki
   cargo test -p anki
   cargo clippy -p anki
   cargo clippy -p anki --target wasm32-unknown-unknown
   (cd cargo/format && cargo fmt --manifest-path ../../rslib/Cargo.toml -- --check)
   ```
   If a commit touched one of the files with a native/wasm split
   (`sync/http_client/{io_monitor,full_sync}.rs`, `sync/collection/
   protocol.rs`, `sync/media/protocol.rs`, `editor.rs`), double-check the
   wasm32 arm got the equivalent change too - upstream's diff will only ever
   touch the single (native) implementation that exists there.

5. **Promote.** Once `next` is green:
   ```
   git checkout main
   git merge --ff-only next
   git push
   ```
   Then update the "last reviewed" line above to the newest commit you
   actually reviewed (not just the newest one you picked - record commits you
   deliberately skipped too, so the next pass doesn't re-review them).

6. Delete or leave `next` for the following round; it gets reset with
   `git branch -f` next time regardless.
