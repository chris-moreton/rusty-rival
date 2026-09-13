# NET-1291 round 3: a single reduced scout delays the winning continuation

On the independent R2-093 critical position, restoring two plies to **one Ka2
search call** changes Rival's one-million-node result from drawing Kb5 (+36)
to winning Kb3 (+141). The control and intervention traces are identical for
53,743 lines before that call. This establishes a concrete causal search event
for this reproducer, rather than inferring a cause from global ablations.

It is diagnostic evidence, **not a general engine fix or a strength result**.
The intervention selects an exact node counter and is deliberately confined
to an archived diagnostic binary. Production source and main are unchanged.

## Position and scope

```
8/8/PR5p/4k1p1/2K3r1/8/8/8 w - - 0 61
```

The [round 2 report](net1291-round2-resource-diagnosis.md) records the frozen,
evaluation-blind sample and exact seven-piece tablebase evidence: Kb3 is the
only winning evasion; Kb5 draws. No new sample, training data, strength match,
or reserved confirmation position was used in this round.

After **61.Kb3 Rg1**, the useful continuation is **62.Ka2 Rg4 63.Rb5**.
Ka2 is a quiet king move outside check; it is distinct from the original
Kb3 check evasion. Root moves themselves are not late-move reduced here.

## Observed control path

All searches use fresh FEN/history, one thread, Hash 128 and a fixed node
budget. Scores below identify their side or window explicitly.

| Event | Evidence |
| --- | --- |
| Iteration 16, after Kb3 Rg1 | White searches depth 13 with window [36,37]. |
| Ka2 reduced scout begins | Node counter 457126; reduction 2; Black child depth 10 instead of unreduced 12; window [-37,-36]. |
| TT shortcut | A depth-10 Lower entry scores -36 for Black, with move Rg4. It closes the child's window. |
| Scout returns | One node consumed; White receives +36, equal to alpha. The full-depth verification condition (`score > alpha`) is false. |
| Completed iteration 16 | At 473090 nodes, Rival keeps Kb5, +36. |
| Iteration 17 finds the continuation | Ka2's depth-11 reduced scout returns -37 for Black and is verified at depth 13. Its White score now beats alpha. |
| Aspiration retries | The Kb3 root call completes with White lower bounds +61 at node 819385 and +111 at node 890478. |
| Budget exhaustion | The next, wider retry is unfinished at one million nodes. The engine returns its last completed iteration: Kb5, +36, depth 16. |
| With more budget | Iteration 17 completes at 1175103 nodes with Kb3, +134; the three-million-node result is Kb3, +137, depth 20. |

The interruption does **not** turn an unfinished score into an exact result:
`current_best` intentionally retains the last fully completed iteration.
Likewise, the depth-10 TT bound meets the requested reduced depth. This trace
does not prove that either condition violates its contract. It shows how
reduction, cached bounds and iteration completion combine to delay this win.

## Single-call counterfactual

A disabled-by-default diagnostic gate changes only the search at node 457126,
ply 3, depth 10: it requests depth 12. There is no global LMR, null-move or
TT change. The exact node counter is an experimental selector, never a
proposed production condition.

The depth-10 TT entry cannot close the deeper search. That search finishes at
node 478372, returning -37 for Black through `Rg4 Rb5 ...`. White's +37 beats
alpha, so the normal scout-verification path continues. The winning move is
now accepted in completed iteration 16 at 779764 nodes (+141), and iteration
17 completes at 820563 nodes. At one million nodes the final choice is Kb3,
+141, depth 17.

| One-million-node result | Move | Score (White) | Last completed depth |
| --- | --- | --- | --- |
| Production | Kb5, draw | +36 | 16 |
| Final trace build, gate off | Kb5, draw | +36 | 16 |
| One Ka2 call searched two plies deeper | Kb3, win | +141 | 17 |

The first 53,743 trace lines match byte-for-byte. The gate fires exactly once.
The subsequent tree changes through normal TT/history updates and search
ordering; this is the intended consequence of the intervention, not evidence
that other positions would benefit from the same extra work.

## Validation and archived evidence

- Base investigation commit: `9172a81f339ec598ef431a09110360d372f9dd8d`.
- Final diagnostic binary SHA256: `9993890dae25ba80d9040fe8d425e133cfbe40f50e34eb763eb4734479e2145f`.
- Feature: `search-width-diagnostics`; logging controlled by `RIVAL_TRACE_PLY`,
  intervention by `RIVAL_TRACE_INTERVENTION=1`. Default intervention is off.
- Gate-off bench: **1,772,650**, identical to production.
- At both 1m and 3m, final gate-off trace matches production's best/ponder move,
  score, completed depth, seldepth, node count, hashfull and PV. Time/NPS are
  deliberately excluded: logging costs time, and these are fixed-node tests.
- Formatting and release build passed. Source is restored after building;
  no production test or suite fixture is changed. No full strength acceptance
  is claimed from a diagnostic build.

[Verification JSON](net1291/round3/verification.json),
[build identity](net1291/round3/build.json),
[temporary patch](net1291/round3/instrumentation.patch),
[build script](net1291/round3/trace.py.txt),
[UCI runner](net1291/round3/run.py.txt),
[archive and verification script](net1291/round3/archive.py.txt), and
[artifact hashes](net1291/round3/sha256.json) are preserved alongside the full
compressed traces and UCI outputs in [round3](net1291/round3/).

Scripts record the investigation's local paths. Locally, the binary and
runnable scripts are in `/home/chris/benchmark/net1291-round3/`. Reconstructing
elsewhere requires adapting those paths. `trace.py` requires clean engine
source, temporarily instruments it, builds the separate binary, and restores
`search.rs` in a `finally` block. The patch is an alternative record of that
instrumentation; do not apply it and then run the clean-source build script.

For a direct UCI reproduction, launch the final diagnostic binary with
`RIVAL_TRACE_PLY=4` and `RIVAL_TRACE_INTERVENTION=0` (control) or `1` (one-call
intervention), then send:

```text
uci
setoption name Threads value 1
setoption name Hash value 128
ucinewgame
isready
position fen 8/8/PR5p/4k1p1/2K3r1/8/8/8 w - - 0 61
go nodes 1000000
```

Wait for `uciok`, `readyok` and `bestmove` as appropriate; only then send
`quit`. Logging goes to stderr and UCI results to stdout. The archived runner
performs that handshake.

## Implication for the next candidate

This narrows the search question to a verified reduced-scout false negative
on the Ka2 continuation, with an additional delay from completing aspiration
retries. It does not establish a position-independent rule for deciding which
fail-low scouts deserve more depth. Blanket removal of LMR, special treatment
of this FEN, or promotion of the previously nonpassing A/C arms would not
follow from the evidence.

A separate general hypothesis is whether to retain a **completed root
fail-high result** when its wider retry is interrupted, while keeping bound
and completed-depth reporting honest. The trace supplies a concrete trigger
for examining that policy; no such production change or strength claim has
been made here. Any candidate still needs the ticket's independent strength,
scaling, suite and reserved-sample acceptance checks.
