# NET-1148: early search-divergence audit

This audit follows the rated classical game `BgmRePNr` and uses the preserved
positions in `tests/data/net1145_closed_sicilian.tsv`. Scores below are from
White's point of view.

## Reproduction contract

- Rusty Rival source: `5e274b0c36d2b5b89aed722f191357db41e3a25a`
- Rusty Rival release binary SHA-256:
  `1cd68b28df265d6b2167294653552445c7e860de01b7dfe050c9d1424dc2f99e`
- Official Stockfish 18 AVX2 binary SHA-256:
  `6b087694916228c905a5e14db74cca8c7e5643602226af1fa5d42353c455b9f9`
- One thread and 64 MiB hash per engine
- `ucinewgame`, `isready`, and a cleared hash before every independent row
- Only exact scores from completed depth iterations are accepted

Example:

```sh
python3 scripts/eval_attribution.py \
  --rival target/release/rusty-rival \
  --stockfish /path/to/stockfish-18-avx2 \
  --positions tests/data/net1145_closed_sicilian.tsv \
  --depth 16
```

## Completed-depth curves

`gap` is Rival's searched score minus Stockfish's searched score.

| checkpoint | raw NNUE gap | d8 | d10 | d12 | d14 | d16 | d18 | d20 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| after 12.exd5 | +29 | +81 | +106 | +94 | +89 | +84 | +81 | +78 |
| after 15.g4 | -4 | +87 | +28 | +21 | +8 | +19 | +30 | +42 |
| after 17.Nf3 | +51 | +144 | +100 | +104 | +107 | +105 | +94 | +106 |

The 15.g4 gap largely collapses with depth, so it is a shallow horizon rather
than evidence that Rival statically overvalues the pawn push. At 12.exd5 both
engines choose 12...Nd4 throughout the deeper curve but disagree about the
position's value. At 17.Nf3, Stockfish consistently prefers 17...f5 while
Rival varies among plausible moves.

## Forced-move control at 17.Nf3

At depth 20, forcing `17...f5` gives Rival a White-POV score of about +80 cp,
only 15 cp worse than its unrestricted +65 cp choice. Stockfish scores the
forced line about -37 cp for White. The cross-engine gap therefore survives
when both engines analyze the same root move: Rival is not failing to generate,
order, or search Stockfish's move.

## Conclusion

No narrow search correction is supported by this game. The early 15.g4 signal
is transient. The persistent 17.Nf3 discrepancy is already partly present in
raw evaluation and remains mostly a valuation/continuation difference when the
root move is controlled. Changing pruning or move ordering from this sample
would be overfitting; an evaluation hypothesis would need an independent
thematic set before any training change.

The investigation did expose a reproducibility defect in the original tool:
clearing the transposition table did not reset persistent search histories
between unrelated rows. Scores changed when the input was subset or reordered.
The corrected tool resets each engine with `ucinewgame` and records the selected
move and PV alongside every exact score.
