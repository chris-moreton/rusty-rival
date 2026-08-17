# SPSA LMP contract

`LMP_MOVE_THRESHOLDS` is duplicated intentionally in Rusty Rival and in the
SPSA seed and fallback tables in chess-compete. The duplication is guarded by
the `spsa-lmp-contract` job in Rusty Rival CI.

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

The job is advisory CI, not a merge gate: neither repository currently has a
branch-protection rule or ruleset requiring its green status. Administrators
can therefore merge or push despite a red job. That is a governance risk to
keep visible, not evidence that the contract ran successfully.

For a coordinated change, merge the chess-compete seed/fallback update first,
then merge the Rusty Rival threshold change. Locally, point the script at the
exact engine checkout under review; it errors rather than skipping when that
path is absent or malformed.

There is a short coordination cost: after the chess-compete change reaches
`main` and before the matching Rusty Rival change merges, every Rusty Rival CI
run sees a mismatch and goes red, including unrelated PRs. Conversely, because
the job reads `chess-compete@main` at run time, re-running an old Rusty Rival
workflow can change from green to red after chess-compete moves. The contract
is one-directional: chess-compete-only changes are checked by its hermetic
tests, while the live-engine comparison runs when Rusty Rival CI runs.
