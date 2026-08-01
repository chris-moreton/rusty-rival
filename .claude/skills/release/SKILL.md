---
name: release
description: Create and publish a rusty-rival release or release candidate — version bump, tag, GitHub binary build, release notes, and verification. Use when asked to cut a release, ship a version, publish an rc, or withdraw a bad release. Not for the local `engines/` binaries (that is /version).
---

# Releasing rusty-rival

Publishes a tagged release with six platform binaries built by GitHub Actions.

**This is not the same as `/version`.** That command copies a compiled binary into `engines/vNNN-name/` for local benchmarking. This skill produces a *public GitHub release* (`vX.Y.Z` tag + built binaries). Don't conflate them.

## Version numbering

`X.Y.Z` for finals, `X.Y.Z-rcN` for candidates. `scripts/release.py` rejects anything else.

**Release candidates belong to the NEXT unreleased version.** Once v1.0.48 has shipped, the next candidates are `1.0.49-rc1`, `1.0.49-rc2`, … — *not* more `1.0.48-rc`. This has been got wrong before.

Check what already exists before choosing:

```bash
gh release list --limit 10
git tag | sort -V | tail -5
```

## Before you tag

Non-negotiable gates. A release is public and awkward to retract.

1. **Full test suite green**, in release mode:
   ```bash
   RUST_MIN_STACK=67108864 cargo test --release
   ```
   ~179 tests across 20 binaries. Release mode matters — debug builds stack-overflow on the search tests.

2. **Bench signature recorded**:
   ```bash
   printf 'bench\n' | ./target/release/rusty-rival | tail -5
   ```
   Deterministic fixed-depth node count. It **must not move** for a pure refactor or a change gated on wall-clock time; it **must** move for any real search/eval change. A signature that changes when you didn't expect it is a bug, not a curiosity.

3. **Working tree clean.** `release.py` refuses to run otherwise (it tolerates only `Cargo.toml`).

4. Never hand-edit the version in `Cargo.toml`. Always `scripts/release.py`.

## The sequence

```bash
python3 scripts/release.py 1.0.49-rc1      # --dry-run to preview
```

It validates the format, refuses if the tag exists, updates `Cargo.toml`, commits `Release vX.Y.Z`, and creates the tag. **It does not push.**

### ⚠️ The Cargo.lock trap — happens every single time

`release.py` commits **only `Cargo.toml`**. The pre-commit hook then runs clippy, which regenerates `Cargo.lock` with the new version — leaving it dirty and, worse, leaving the tag pointing at a commit whose lockfile disagrees with its manifest.

Fix it *before pushing*, while it's still cheap:

```bash
git status --porcelain                 # expect: M Cargo.lock
git add Cargo.lock
git commit --amend --no-edit
git tag -f vX.Y.Z                      # retag: amend changed the SHA
git status --porcelain                 # must now be empty
git rev-parse --short vX.Y.Z HEAD      # the two must match
```

Amending after pushing means a force-push. Amending before costs nothing. Always check.

### Push

```bash
git push origin main
git push origin vX.Y.Z
```

## Watching the build

The tag push fires **two** workflows: `Continuous integration` (fast) and `Release` (~5 min, builds the binaries).

**Filter on the workflow name.** Matching the tag branch alone grabs the CI run, and you will then 404 looking for assets that were never built:

```bash
gh run list --limit 5 --json workflowName,status,conclusion,headBranch,databaseId \
  --jq '.[] | "\(.workflowName) | \(.headBranch) | \(.status) \(.conclusion // "") | id=\(.databaseId)"'

gh run view <RELEASE_RUN_ID> --json status,conclusion
```

Expect six assets: linux-x86_64 (±avx2), macos-aarch64, macos-x86_64, windows-x86_64 (±avx2).

## Pre-release marking is automatic

`release.yml` sets `prerelease: ${{ contains(github.ref_name, '-') }}` — **any tag containing a hyphen** is flagged a pre-release. So `-rcN` is handled for you, and public `latest` keeps resolving to the most recent final release. Don't set it by hand; do verify it.

## Release notes

The workflow auto-generates a changelog. **For a final release that is not sufficient** — write real notes: what changed, per-change match results, and measured Elo versus the previous release.

```bash
gh release edit vX.Y.Z --notes-file /path/to/notes.md
```

State limitations explicitly rather than only the wins. If a strength claim is unmeasured or a result is within noise, say so in the notes — they outlive the conversation that produced them.

## Verify what the public actually sees

Never assume; the API and the web view can disagree with your intent.

```bash
gh release view vX.Y.Z --json tagName,isDraft,isPrerelease,assets \
  --jq '"tag=\(.tagName) draft=\(.isDraft) prerelease=\(.isPrerelease) assets=\(.assets|length)"'

gh api repos/chris-moreton/rusty-rival/releases/latest --jq .tag_name   # what "latest" resolves to

curl -s -o /dev/null -w "%{http_code}\n" -L \
  https://github.com/chris-moreton/rusty-rival/releases/download/vX.Y.Z/rusty-rival-vX.Y.Z-macos-aarch64
```

Wanted: a final release becomes `latest`; an rc does not; assets return 200.

## After a final release

Two standing rituals, both easy to forget:

### 1. Stockfish anchor ladder

Run from the **`chess-compete` repo, a sibling of this one** — `../chess-compete` (absolute: `~/git/chris-moreton/chess/chess-compete`). It is a **separate git repository**, not a subdirectory, and it has its own virtualenv which must be used — there is no top-level `compete` executable on `PATH`:

```bash
cd ../chess-compete

# 1. Register the new release. --init downloads the macos-aarch64 asset
#    straight from the GitHub release, so publish before running this.
.venv/bin/python -m compete --init rusty vX.Y.Z

# 2. Ladder: 50 games against each anchor.
#    sf-3000 is the ceiling — Stockfish caps UCI_Elo near 3190, so no sf-3200 exists.
.venv/bin/python -u -m compete vX.Y.Z sf-2600 --games 50 --time 0.5
.venv/bin/python -u -m compete vX.Y.Z sf-2800 --games 50 --time 0.5
.venv/bin/python -u -m compete vX.Y.Z sf-3000 --games 50 --time 0.5
```

**Run it detached** — `nohup … > log 2>&1 & disown` — and poll the log. Backgrounded session tasks are killed after ~10 minutes and have destroyed multi-hundred-game runs.

**0.5s/move on a verified-idle machine is mandatory, not a preference.** Afterwards, validate against the PGN `WhiteNPS`/`BlackNPS` headers: Stockfish should read ~0.9–1.1M. Elo-capped Stockfish is roughly node-independent while rusty-rival is not, so CPU contention selectively destroys rusty at the upper rungs and produces convincingly-wrong numbers. A whole v1.0.45 ladder was thrown away to this. Load average alone is a poor gate on this Mac (I/O inflates it badly) — confirm with `top -l 1 -n 0 | grep "CPU usage"` for healthy idle%, plus the NPS headers.

The compete database retains discarded runs, so split any results query by `created_at` — otherwise you pool a rerun with the bad run it replaced.

### 2. Notion

A row in the Verification matches table on the Code Audit page, plus the Ratings page for the ladder results.

Also comment the result and set state on the Linear ticket.

## Withdrawing a bad release

**Convert to a draft. Do not delete.**

```bash
gh release edit vX.Y.Z --draft
```

The release vanishes from the public page, `latest` falls back to the previous final release, and the assets 404 — while the notes and binaries are preserved and the git tag survives for bisecting. One command undoes it (`--draft=false`).

Deleting destroys the notes and built binaries irrecoverably. `gh` will print an `untagged-…` URL after drafting; that is just the draft's private edit URL, not a broken state — confirm with:

```bash
gh api repos/chris-moreton/rusty-rival/releases \
  --jq '.[] | select(.tag_name=="vX.Y.Z") | "draft=\(.draft) assets=\(.assets|length)"'
```

## Gotchas

- **`chess-compete` is a sibling repo (`../chess-compete`), not part of this one.** Both `scripts/ab_match.sh` and `scripts/ab_time_match.sh` hardcode its opening book at `$HOME/git/chris-moreton/chess/chess-compete/openings/8moves_v3.pgn` and abort with `book not found` if it is not checked out. The anchor ladder needs it too.
- **Don't run heavy cargo builds while a match is playing.** CPU contention distorts game timing and invalidates results.
- Long matches must be `nohup … & disown` — backgrounded tasks are capped around 10 minutes and get killed.
- Commit messages: no AI/Claude attribution.
- `RUST_MIN_STACK=67108864` (64MB) for debug test runs; the older 16MB figure now overflows.
