# Rook-ending investigation: 18 September 2026

**Outcome:** the EPD endgame gap persists in public v1.0.68. We reproduced concrete missed resources, checked exact seven-piece answers, and rejected a diagnostic cache-policy change as a demonstrated remedy. No production code, network, release or bot configuration was changed. No Elo claim is made.

## History

[NET-1291](https://linear.app/netsensia/issue/NET-1291) contains the original investigation. Its description ends at an earlier inconclusive test; later comments and commit `530f8a5` show root fail-high retention shipped in v1.0.65. Earlier quiescence-draw, TT-depth and network-continuation experiments were nonpassing. They were not repeated here.

The PGO implementation already in progress remains separate and unmodified by this experiment.

## Fresh comparison

Full EET, fresh engine process per position, one thread, requested Hash 128, no tablebases, concurrency 4 on this Ryzen 5950X. Both peers were built from pinned open-source revisions. These are diagnostic screens under normal desktop activity, not guarded speed measurements or statistical strength estimates. Different engines' nodes, depths and centipawn scales are not interchangeable.

| Engine | 1 second, all 100 | 5 seconds, all 100 | 5 seconds, rook 53 |
|---|---:|---:|---:|
| Public Rival v1.0.68 AVX2 |55|67|30|
| Ethereal 14.40 public HCE |69|79|38|
| Stash 37.26 |72|83|41|

At five seconds Rival solves 25/27 minor-piece and 4/5 pawn positions, versus 26/27 and 4/5 for both peers. The rook and queen gaps remain larger. Fourteen positions are missed by Rival and solved by both peers at five seconds; full identities, moves and scores are in [comparison.json](comparison.json). Peer agreement is not ground truth.

Rival's requested 128 MiB hash allocates 96 MiB of entries. That is reported as practical configured behavior, not equal physical capacity. Hash 96/128/192 sensitivity was checked separately on the localized resource.

## Exact ground truth

All three EET roots with <=7 pieces were queried against the Lichess tablebase API, with full responses preserved in [tablebase-audit.json](tablebase-audit.json). Move categories in the API are from the resulting side-to-move perspective.

- **036:** Kc5 is the only drawing move. Rival chooses Bxf1, which loses.
- **053:** Kh7 is the only winning move. Rival immediately promotes to a queen, which draws.
- **056:** Rxf5+ is the only winning move. Rival misses it at 1 s but finds it at 5 s; Ethereal behaves the same, while Stash already solves it at 1 s.

Stash and Ethereal also miss036/053 at 5 s; these cases establish genuine mistakes but do not explain the gap against those two engines. Stockfish 19 finds all three exact moves at approximately 1M nodes with tablebases disabled and zero reported hits; see [tb-reference.json](tb-reference.json). Node counts are not used to rank speed across engines. A leaf tablebase result reached along one chosen PV does not prove the original root's minimax value.

## Concrete resource: EET049

The eight-piece root is outside the exact root tablebase audit:

`1k6/8/8/1K6/5pp1/8/4Pp1p/R7 w - - 0 1`

All three reference engines select Kb6 and score a draw at 5 s. Rival selects Kc6/-1004. Stockfish 19, searching each forced move with a fresh game/TT at 5 s, scores Kb6=0 and Kc6=-270 in its own centipawn scale. This is reference evidence, not an exact eight-piece proof.

The defensive timing mechanism can be checked directly:

- `Kb6 g3 Kc6 g2 Rb1+ Kc8 Ra1 h1=Q`: promotion **does not check**, because the g2 pawn blocks the queen's diagonal. **Ra8 is mate**.
- `Kc6 g3 Rb1+ Kc8 Ra1 h1=Q`: promotion **does check**, so Ra8 is illegal.

[resource-check.json](resource-check.json) verifies legality/check/mate using python-chess. These illustrative lines explain a real resource; they are not an exhaustive proof against every defense. The relevant theme is quiet king/rook moves controlling promotion checks, not simply a rook material value.

### Localization before and after ...g2

Fresh-position searches, one thread, one million nodes:

| Start | Rival's choice | White-relative score |
|---|---|---:|
| Before ...g2 | ...g2 |-1746|
| Same parent, forced ...g2 | ...g2 |0|
| Resulting child as a fresh root | Rb1+ |0|

Parent: `1k6/8/2K5/8/5p2/6p1/4Pp1p/R7 b - - 1 2`.
Child: `1k6/8/2K5/8/5p2/8/4Pppp/R7 w - - 0 3`.

Results at Hash 96 and128 match; Hash 192 doubles entries but leaves these choices and scores unchanged. Detailed traces and bound flags are in [boundary.json](boundary.json). Thus capacity alone does not resolve this instance. The fresh child gets its whole budget and iterative deepening, whereas an internal child shares a budget and inherits windows, ordering history and TT state. The discrepancy is a localization target, not proof of a particular cutoff bug.

Static NNUE values are pessimistic even where a fresh search finds the draw. That shows search can recover the resource; it does not establish whether training labels, evaluation, pruning, ordering or their interaction caused the original miss.

## Comparison with peer source

- [Stash pinned search](https://github.com/mhouppin/stash-bot/blob/13e0a81a4af7fb6af43405228535c5058ee71657/src/sources/search.c) avoids TT score cutoffs at PV nodes. [Ethereal pinned search](https://github.com/AndyGrant/Ethereal/blob/0e47e9b67f345c75eb965d9fb3e2493b6a11d09a/src/search.c) does so except at its quiescence boundary. Rival permits them on wide windows too.
- Both peer quiescence implementations omit quiet checks. Consequently, adding quiet checks is **not** an explanation of why those peers succeed here.
- Both peers check draws in quiescence; Rival does not. That is already within the earlier nonpassing Arm A experiment's scope, not a new discovery to repackage.
- Stash contains explicit endgame scaling; Ethereal has complexity and material scaling. Their presence does not establish that those terms solve049. No constants were copied or tuned.

## Isolated diagnostic intervention

The archived [patch](diagnostic-source.patch) changes only an isolated source copy. `RIVAL_EPD_DISABLE_PV_TT` suppresses TT score returns and bound narrowing at **incoming windows wider than one**, retaining hash move, static evaluation and singular-extension metadata. This is a window-width proxy, not a new node-type flag. No TT policy changes were applied to the working engine source.

Switch off: **80/80 full normalized fixed-node search traces match public v1.0.68**, including scores, bounds, PVs and nodes. Switch on, at 1M nodes: none of036/049/053/060/081 switches to the expected answer.049 remains Kc6/-1004. The localized parent changes to an underpromotion and a smaller nonzero score; the child remains a draw. The ablation does solve 036 at 100k nodes, then misses it again at 1M: this budget-dependent flip is another reason it is not a remedy. This does not demonstrate a fix or uniquely establish a history-dependent TT error.

All six suites were also run at 100k nodes, with every gained and lost position retained. See [diagnostic-comparison.json](diagnostic-comparison.json). These results are a regression diagnostic, not a strength verdict. A gain in one column cannot justify hiding losses elsewhere.

| Suite,100k nodes | Control | Wide-TT disabled | Newly solved | Newly missed |
|---|---:|---:|---:|---:|
| Arasan18 |55|59|19|15|
| Bratko-Kopec |19|18|0|1|
| EET |39|32|5|12|
| Quick |7|7|0|0|
| STS |890|894|162|158|
| WAC |266|266|9|9|

STS graded points fall from 10,821 to 10,756 out of 15,000 despite four more fully solved positions. All runs report zero errors. WAC illustrates why totals alone are inadequate: an unchanged266 hides nine gains and nine losses. The intervention fails the diagnosed resources and worsens EET; it is **not promoted to a strength candidate**. No SPRT was run.

## What follows from this round

The next useful investigation is a trace of the ...g2 child under the unrestricted parent versus the forced-parent control: requested depth/window, reductions, first searched moves, and the origin/history of any TT score used. A non-draw TT record alone is insufficient to prove a history error; a same-node counterfactual must recover the resource.

Any eventual general change must be tested on independent game-derived resources, report every suite and every regression, preserve the original unused 100-position confirmation set, and pass paired games at short and longer controls. No parameter selection from EPD totals, no position-specific production logic, and no release from this diagnostic round.

## Evidence

The adjacent JSON files contain all selected searches, comparisons and exact responses. `evidence.tar.gz` preserves the runner scripts, full six-suite records, raw UCI boundary trace and the 80-position identity check. `evidence-sha256.json` records archive and binary provenance. No binaries or neural networks are bundled.

Independent read-only review by the existing Claude coder checked the report against the adjacent evidence and pinned sources. The budget-dependent 036 flip and 056 peer behavior were added after that review.

The archive checksum covers the bundled compressed bytes directly (`sha256sum evidence.tar.gz`), not a serialized aggregate of inputs. The original archive is retained unchanged; tar/gzip metadata was not normalized, so byte-identical repacking is not claimed. `archive_members_sha256` lists every regular member in path-sorted order with SHA-256 of its exact uncompressed bytes, allowing content verification independently of archive metadata. The adjacent diagnostic source metadata expands the historical abbreviated base revision to its full Git object ID.
