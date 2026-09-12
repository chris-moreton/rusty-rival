# NET-1291: endgame diagnosis and experiments

Work started from `ce4d3f6` (engine 1.0.64). Baseline binary SHA prefix
`1c66b571`, bench 1,772,650. Five candidates completed 14,137 games without an SPRT improvement pass.
No engine or production-network change is proposed for merge. NET-1291
remains open: the investigation has found evaluator/search interactions and
concrete search omissions, but no general cause of the entire suite gap or
accepted strength improvement.

| Experiment | Games | Elo ± 95% error | Verdict |
|---|---:|---:|---|
| A: qsearch draw policy | 4,000 | +4.9 ± 9.3 | Inconclusive at cap |
| B: qsearch material draws | 4,000 | +1.0 ± 9.3 | Inconclusive at cap |
| C: TT depth units | 3,941 | −1.8 ± 9.4 | H0 accepted |
| D: 25% result-target continuation | 914 | −24.8 ± 19.7 | H0 accepted |
| D control: original 75% target | 1,282 | −15.7 ± 16.4 | H0 accepted |

H0 acceptance means failure of the required improvement test; it does not
by itself prove a strength loss. All six suites were completed for every
candidate. No nonpassing candidate advanced to the longer match, ladder or
reserved rook sample. The Lichess bot was restarted and verified active.

## Baseline evaluator comparison

EET, 5 seconds per position, one thread, Hash 128, concurrency 4. The bot was
stopped between games and restarted afterwards. NNUE numbers are the existing
matching cached baseline; HCE is a fresh run of the same binary with
`UseNNUE=false` (option hash `16c26dc8`).

| Material class | Positions | NNUE solved | HCE solved | NNUE mean depth | HCE mean depth |
|---|---:|---:|---:|---:|---:|
| Pawn | 5 | 4 | 3 | 26.2 | 25.4 |
| Minor | 27 | 24 | 12 | 28.4 | 27.1 |
| Queen | 15 | 7 | 7 | 25.7 | 25.8 |
| Rook, including minors | 53 | 25 | 11 | 23.6 | 24.5 |
| Total | 100 | 60 | 33 | | |

HCE is not a stronger endgame evaluator on this diagnostic. Its slightly
higher depth in rook positions does not translate to better solutions.

## Training provenance correction

The ticket's account of the production net's training data is incorrect.
`src/nnue.rs` documents 600 superbatches over the corrected **Stockfish
depth-9** corpus. NET-1095 and chess-compete's training scripts corroborate
that provenance. `src/datagen.rs` describes a different data-generation path;
its 1,000 cp / six-ply adjudication cannot be blamed for this net's errors.

The local corrected depth-9 corpus in `~/net53/data` has 35 shards and
512,363,260 records. A diagnostic sample of 10,000 equally spaced records per
shuffled shard yielded:

| Output bucket | Sample positions | Positions containing a rook and no queen |
|---|---:|---:|
| 0 (2–5 pieces) | 50,560 | 28,126 |
| 1 (6–9) | 62,908 | 38,778 |
| 2 (10–13) | 53,342 | 34,070 |
| 3 (14–17) | 44,539 | 25,440 |
| 4 (18–21) | 39,635 | 16,730 |
| 5 (22–25) | 38,869 | 9,202 |
| 6 (26–29) | 40,990 | 2,887 |
| 7 (30–32) | 19,157 | 86 |

These are equal-shard sample counts, not estimates weighted by shard size.
This local corpus has not been proven byte-identical to NET-1095's input.
Nevertheless, absence of low-material/rook examples is not supported by this
sample; coverage quality and depth-9 labels remain open questions.

## Equal-time searches and PV walks

Five pre-change examples were examined: EET 036, 049, 060, 063 and 081.
Fresh root searches used 5 seconds, one thread and Hash 128 for NNUE, HCE,
Stash and Stockfish. Both Rival evaluators were then evaluated statically at
every node of Rival's reported PV. Stockfish's static output and fresh 200 ms
searches were recorded at those nodes. Root moves from the EPD were separately
forced for all three evaluators, and Rival was queried along Stockfish's PV.
These short node searches are diagnostic evidence, not ground truth or
directly comparable calibrated scores across engines.

* 036: Rival prefers capturing a rook and entering a difficult minor-versus-
  pawn ending; Stockfish maintains a draw with a king walk. Forcing the king
  move does not make either Rival evaluator recognize the draw at 5 seconds.
* 049: both Rival evaluators miss the drawing king move in unrestricted
  search. With that move forced, HCE finds a draw, NNUE still scores -298.
  Along Stockfish's line, NNUE evaluates the drawn checking position after
  four plies at -672, but its search already recognizes the draw in 200 ms.
  Earlier in the line it still chooses a different move. This is an
  evaluation/search interaction, not evidence for blanket material scaling.
* 060: NNUE chooses a promotion sequence into queen versus rook, valuing the
  resulting long checking/shuffling line at about +485. Stockfish's short
  searches along much of that line trend toward zero; it prefers preserving
  the advanced pawn while moving the king out of the way. HCE overvalues
  the queen-versus-rook alternative still more strongly.
* 063: both Rival evaluators undervalue the defensive pawn advance and the
  subsequent defensive setup. Rival recognizes a draw further along the
  peer's line but does not find the same quiet defensive moves earlier.
* 081: even forcing the correct rook move gives Rival a draw. Its subsequent
  PV trades bishops and drifts into a drawn continuation, while peers
  maintain a winning evaluation. The root static NNUE value is already
  positive (+681), so this is not simply a missing root winning bonus.

The selected final PVs and node evaluations are committed in
[`net1291/diagnosis.json`](net1291/diagnosis.json) and
[`net1291/controls.json`](net1291/controls.json). Full transcripts and peer
static-evaluation dumps are retained locally under
`~/benchmark/net1291-{diagnosis,controls}.json`. These observations do not
justify fitting a scaling factor, rook bonus or king bonus to the suite.

## Experiment A: draws inside quiescence

General statement: **Tactical search must recognize draws reached during its
continuations, rather than assigning them an NNUE stand-pat score.**

Code inspection found that quiescence omitted both draw detection and child
repetition-history updates. Full search has both. Independent synthetic tests
reproduced a reversible check evasion across the fifty-move boundary scored
+3 instead of zero, and an already-repeated rook position scored +155 instead
of zero. A checkmate-at-the-boundary control passes on the baseline.

Arm A applies the existing full-search draw policy in quiescence and balances
history push/pop around its legal children. It adds no tuned constants. This
is a confirmed omission, but its contribution to the rook-ending strength gap
is **unproven**. Game testing decides whether this implementation is accepted.

The three new tests pass with the patch and the complete
`RUST_MIN_STACK=67108864 cargo test --release` run passes. Arm A's binary is
`~/benchmark/sprt/rival-net1291A`, SHA256
`01386680304a1a73afb2eaa957149b5b73776cceb5f6ffb54a8b2b27455234d4`.
Its bench is 1,625,045 (not node-identical to baseline). Bench node reduction
is not a strength result. Protocol S [0,10] at 1+0.01 ended **inconclusive** at
the 4,000-game cap: 1,511 wins, 1,455 losses, 1,034 draws, **+4.9 ± 9.3 Elo**,
LOS 84.8%, LLR -0.0613. No time losses/crashes were reported. Recomputed paired
counts were 235 / 377 / 738 / 397 / 253, paired 95% interval [-4.0,+13.8].
Neither boundary was reached; this is not a pass and Arm A is not accepted.
The exact source change is retained as [`net1291/arm-a.patch`](net1291/arm-a.patch).

All-suite diagnostic at 300,000 nodes, one thread, Hash 128:

| Suite | Baseline | Arm A | Arm B | Arm C |
|---|---:|---:|---:|---:|
| arasan18 | 59/250 | 67/250 | 59/250 | 68/250 |
| bratko-kopec | 20/24 | 20/24 | 20/24 | 19/24 |
| eet | 43/100 | 44/100 | 43/100 | 42/100 |
| quick | 7/10 | 7/10 | 7/10 | 7/10 |
| sts | 967/1500; 11,364/15,000 points | 980/1500; 11,502/15,000 points | 967/1500; 11,364/15,000 points | 959/1500; 11,299/15,000 points |
| wac | 272/300 | 271/300 | 272/300 | 270/300 |

Every suite completed without runner errors. These fixed-node scores are
diagnostics only; they do not supersede the inconclusive game result. Arm A
did not advance to the 10+0.1 check or reserved sample.

## Experiment B: material draws inside quiescence only

General statement: **Tactical exchanges must not assign a winning stand-pat
evaluation to material that full search treats as drawn.**

This ablates Arm A down to its material-draw check, starting again from the
unchanged 1.0.64 baseline. It uses full search's existing `ply > 6` and
`insufficient_material(..., true)` policy. It does not add repetition-history
maintenance or fifty-move handling. Independent synthetic positions check
drawn material and preserve bishop-pair / bishop-and-knight mating material.
The complete release test suite and workspace clippy pass. Its bench remains
1,772,650; this does not establish general node identity or strength.
Protocol S [0,10], 1+0.01 ended **inconclusive** at 4,000 games:
1,497 wins, 1,486 losses, 1,017 draws, **+1.0 ± 9.3 Elo**, LOS 58.0%,
LLR -1.8. No strength gain or acceptance is claimed. Its source change is
retained as [`net1291/arm-b.patch`](net1291/arm-b.patch).
All six 300k-node suite scores match the baseline (table above); no runner
errors. No scaling, ladder or holdout run was warranted. Binary SHA256:
`546fbee46eae628aa0426c7ba113b330e7d1cc26463cf75488be3e7b09a67131`.

## Experiment C: use consistent depth units in the transposition table

General statement: **A cached search of a checked position must not satisfy a
request one ply deeper merely because both visits extend the check.**

This known omission is independently described in NET-1251 (still backlog).
Code inspection confirms that the probe compares against incoming depth but
the three store paths use check/singular-extended depth. Therefore an in-check
node searched with incoming depth 3 can satisfy a later incoming-depth-4
request, and a one-ply-reduced checking move can bypass its intended full-depth
verification. This directly concerns the long checking lines found during
the rook-ending PV diagnosis, though its strength contribution is unproven.

The isolated candidate starts from the original baseline, keeps actual search
depth and history bonuses unchanged, and stores incoming depth on every TT
store path. It contains neither draw-handling arm. An independent synthetic
rook-check test checks the stored depth and requires a deeper revisit to
search past the root. The baseline fails (stored 4, expected 3); the candidate
passes, including both cold and seeded-hash cutoff paths. The complete release
suite passes (229 tests, 3 ignored), as does workspace clippy. Its bench is
2,050,657, versus 1,772,650 baseline. Binary SHA256:
`a7393247499ae0641e8b046946338062ceb996e9b242a1e40cb4acedeff21606`.
Protocol S [0,10], 1+0.01 **rejected the candidate (H0 accepted)** at
3,941 completed games: 1,468 wins, 1,488 losses, 985 draws,
**−1.8 ± 9.4 Elo**, LOS 35.6%, LLR −2.94. The eleven games still in
flight were cancelled when the bound was reached; they are recorded as
“No result” and excluded from the 3,941 completed games. No time losses
or engine crashes were reported. This does not establish a strength loss,
but it fails the required improvement test.

All six 300k-node suites completed without errors: arasan18 68/250,
bratko-kopec 19/24, eet 42/100, quick 7/10, sts 959/1500 with
11,299/15,000 points, wac 270/300. No scaling, ladder or reserved-sample
run followed. The patch and regression tests are preserved in
[`net1291/arm-c.patch`](net1291/arm-c.patch) and
[`net1291/arm-c-tests.rs.txt`](net1291/arm-c-tests.rs.txt); the engine source
was restored to the unchanged baseline. NET-1251's correctness question
remains separate from this ticket's strength requirement.

## Verified checkpoint for a possible training experiment

The original optimiser checkpoint was downloaded from
`s3://chess-compete-builds/nnue-checkpoints-sf/rival-512x2-ob8-corrected-net1095-600/`
to `~/benchmark/net1291-training/parent`. Its quantised export has SHA256
`27b35937dfa3d221ae5fb1d8e31f41b713f7e77cdae0b4efb93547197b4b70ff`,
exactly matching the embedded production network. This verifies the parent
network, not the identity of the local training corpus. Reloading and exporting with the pinned Bullet revision reproduced these
bytes exactly. No new network has been accepted.

## Reserved generalisation sample

Before evaluating a candidate, 100 rook endings were sampled from the
pre-existing `~/benchmark/sprt/net1284-scale.pgn`. Selection takes the first
eligible position per game after 40 plies, with 6–12 pieces, rooks on both
sides, no queens/minors, and excludes positions matching repository suites.
Eligible candidates were sampled with seed 1291. Scores and outcomes were
not used. The frozen manifest is `~/benchmark/net1291-holdout.json`, SHA256
`81bd6c5e32195a2738128d6e599b6ec5232f58348f00ae4d28d2c6ae16e790cf`.
The manifest is also retained in [`net1291/holdout.json`](net1291/holdout.json).

## Registered training pilot D — result/score target blend

Hypothesis (unproven): weighting game outcomes at 75% can obscure difficult
endgame resources that the teacher score sees. Both targets are noisy: the
Stockfish labels are only depth 9. This is a general target-calibration
experiment, not an asserted explanation of the five diagnosis examples.

Two independent continuations start from the exact production optimiser
checkpoint and use the same sorted 35-shard local corpus. The control retains
75% result / 25% score targets; the candidate uses 25% result / 75% score.
Each runs five superbatches of 6,104 × 16,384 positions (500,039,680 total,
approximately one pass through the local corpus), cosine learning rate
0.0001 to 0.00001, unchanged architecture, AdamW and evaluation scale 400.
Both restart from the parent, including optimiser state. Only the final
checkpoint is eligible; suite performance will not select checkpoints.

The corpus is reused as-is; no diagnosis or reserved positions are added.
The control separates a target-blend change from generic continuation effects.
Training losses across different target blends are not comparable strength
metrics. Both final nets will face the existing Protocol S acceptance gate.
If neither passes, this pilot ends without scaling or holdout evaluation.
The exact trainer, dependency lockfile and pre-run manifest are retained in
[`net1291/training/`](net1291/training/).

### Pilot D training and export checks

Both arms completed: control 136.8 seconds, candidate about 127 seconds.
Different-target training losses are recorded solely for reproducibility.
Each exported net is 803,904 bytes. Exact network and engine binary SHA256s
are retained in [`training/artifacts.json`](net1291/training/artifacts.json).
The full local shard set and parent checkpoint files are hashed in the manifest.

Each candidate passes 226 release tests with three existing ignores. The one
excluded test, `nnue_golden_values_match_net1095`, intentionally asserts exact
outputs of the old production weights; it first failed on the control, as
expected when weights change. It remains untouched in the repository. No
other test is excluded. Both new networks independently match a scalar
quantised reference on all 254 positions (the eight original golden FENs plus
deterministic random legal play, seed 129102). Loading/overflow, perspective,
material-sign, bucket selection and incremental accumulator checks pass.
If a network is accepted, the golden test must be deliberately regenerated
and the complete unfiltered suite rerun before merging.

Both candidates completed Protocol S, candidate first then control, with
opening RNG seed 1291. All six 300k-node suites followed each match. There is
no engine search change in either binary. The worktree's embedded-network
reference was restored to production before starting games.

### Pilot D candidate — rejected

The 25%-result candidate crossed H0 at 914 completed games: **315 wins,
380 losses, 219 draws; −24.8 ± 19.7 Elo**, LOS 0.7%, LLR −2.96.
Bench is 2,513,915 versus 1,772,650 baseline. All six 300k-node suites
completed without errors: arasan18 74/250, bratko-kopec 18/24, eet 38/100,
quick 7/10, sts 982/1500 (11,582 points), wac 274/300. The improvement
test rejects this candidate; no scaling, ladder or reserved-sample run follows.
A short fine-tune is not a universal test of optimal training-target weights,
but it provides no support for this proposed change. The 75%-result control
was measured separately against baseline, as reported below.

### Pilot D control — rejected

The original-target continuation also crossed H0: **448 wins, 506 losses,
328 draws over 1,282 games; −15.7 ± 16.4 Elo**, LOS 3.0%, LLR −2.96.
Bench is 1,990,904. Each D match cancelled eleven in-flight games at its
stopping bound; these are excluded from the completed counts. Neither match
reported time losses or engine crashes.

| Suite, 300k nodes | Baseline | D candidate (25%) | D control (75%) |
|---|---:|---:|---:|
| arasan18 | 59/250 | 74/250 | 68/250 |
| bratko-kopec | 20/24 | 18/24 | 18/24 |
| eet | 43/100 | 38/100 | 41/100 |
| quick | 7/10 | 7/10 | 6/10 |
| sts | 967/1500; 11,364 points | 982/1500; 11,582 points | 960/1500; 11,410 points |
| wac | 272/300 | 274/300 | 271/300 |

All suites finished without errors. Neither network is accepted. Since the
unchanged-target continuation also failed, the measured loss cannot all be
attributed to target blending. These separate baseline matches do not
establish the direct Elo difference between the two new nets. Any retry
should first establish a stable continuation control and state how its data
or optimisation assumptions differ. No checkpoint was selected by suite score.

## Remaining investigation

The historical experiment register (NET-1149) rules out repeating blanket
rule-50 tapering or hand-picked material scaling without a materially new
mechanism. The earlier insufficient-material experiment (NET-230) applied
its guard on every NNUE evaluation; A/B instead tested the existing full-search
draw policy at quiescence entry, with no accepted gain. NET-1160 added dynamic
null-move reduction, but deliberately deferred verification pending targeted
evidence. No verification patch was added here on speculation.

The next diagnosis needs independent positions that reproduce a specific
missed resource, followed by controlled search instrumentation or deeper
teacher-label checks. Neither another arbitrary search constant nor more
training on the same assumptions is justified by these results. The 100
reserved rook endings remain unused for eventual confirmation.

## Reproduction and local records

Archived scripts have a `.txt` suffix and retain this workstation's paths.
Copy them into `~/benchmark/` with the original `net1291-` prefix (training
scripts into `~/benchmark/net1291-training/`) to repeat the commands. Python
chess is available via `~/services/lichess-bot/.venv/bin/python`. The idle
window wrapper must use the normal Python interpreter; it checks both local
engine processes and Lichess's public playing state before stopping the bot
and restarts the service in `finally`. Never build during benchmark matches.

Full game records and match logs are `~/benchmark/sprt/net1291{A,B,C}.{pgn,log}`.
Training pilot game records use `net1291D-{candidate,control}` instead. Full
EPD result JSONs remain untracked under `epd/results/rusty-rival/`, selected
by their explicit binary SHA. They are not committed into the live result
store: all experimental binaries report version 1.0.64, and multiple records
with that label would make CI's baseline selector ambiguous. The tables above
report every suite; exact diagnostic PVs and sampled-data counts are archived
alongside this report. Patches contain ordinary unified-diff blank context
lines (a single space), which a blanket whitespace check can flag.
