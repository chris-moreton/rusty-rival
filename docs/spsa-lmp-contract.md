# SPSA LMP contract

`LMP_MOVE_THRESHOLDS` is duplicated intentionally in Rusty Rival and in the
SPSA seed and fallback tables in chess-compete. The duplication is guarded by
the required `spsa-lmp-contract` job in Rusty Rival CI.

The job checks out the Rusty Rival commit being built at `GITHUB_WORKSPACE` and
checks out `chris-moreton/chess-compete@main` at `chess-compete`. It runs:

```bash
python3 chess-compete/scripts/check_lmp_contract.py --engine-root "$GITHUB_WORKSPACE"
```

This deliberately uses chess-compete's default branch rather than a cached or
event-derived ref. An engine-only PR that changes an LMP threshold therefore
fails until the matching chess-compete update is on `main`; it cannot pass by
testing a stale sibling checkout. Both repositories are public, the checkout
uses no secret or write token, and the job runs for fork pull requests as well.

For a coordinated change, merge the chess-compete seed/fallback update first,
then merge the Rusty Rival threshold change. Locally, point the script at the
exact engine checkout under review; it errors rather than skipping when that
path is absent or malformed.
