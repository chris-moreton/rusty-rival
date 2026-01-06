# Java Rival vs Rusty Rival: Engine Comparison

This document compares the search and evaluation implementations of the Java/Kotlin engine (rivalchess-engine v36) with the Rust engine (rusty-rival v029).

## Overview

| Aspect | Java Rival (v36) | Rusty Rival (v029) |
|--------|------------------|-------------------|
| Language | Kotlin | Rust |
| Speed (NPS) | ~500K-1M | ~3-7M |
| Elo (estimated) | ~1550-1600 | ~1520-1560 |

Despite being 5-10x slower in raw node throughput, the Java engine performs competitively. This suggests it has superior evaluation and/or search selectivity.

---

## Search Algorithm Comparison

### Core Algorithm

Both engines use the same fundamental approach:
- Iterative deepening
- Alpha-beta with aspiration windows
- Principal Variation Search (PVS/Scout)

### Search Enhancements

| Enhancement | Java Rival | Rusty Rival |
|-------------|-----------|-------------|
| Transposition Table | Yes | Yes |
| Null Move Pruning | Yes | Yes |
| Late Move Reductions (LMR) | Yes (with history score) | Yes (logarithmic formula) |
| Killer Moves | Yes (2 per ply + mate killer) | Yes (2 per ply) |
| History Heuristic | Yes (success/fail ratio) | Yes (depth-weighted) |
| Check Extensions | Yes (fractional) | Yes (full ply) |
| Threat Extensions | Yes (from null move) | No |
| Pawn Push Extensions | Yes (fractional) | No |
| Internal Iterative Deepening (IID) | Yes | Yes |
| Futility Pruning | Yes (depth 1-3) | Yes (alpha pruning) |
| Reverse Futility Pruning | No | Yes (beta pruning) |
| Static Exchange Evaluation (SEE) | Yes | Yes |
| Delta Pruning (quiescence) | Yes | No |
| Quiescence Search | Yes | Yes |

### Key Differences in Search

**Java Rival has additional pruning/extensions:**
- **Threat Extensions**: If null move search reveals a threat (opponent's best reply is very good), extend the search
- **Pawn Push Extensions**: Extends for advanced pawn moves (potential promotions)
- **Delta Pruning**: In quiescence, prunes captures that can't possibly raise alpha even with the largest piece gain
- **Fractional Extensions**: Accumulates partial extensions and only extends when they sum to a full ply

**Rusty Rival differences:**
- **Logarithmic LMR**: Uses `ln(depth) * ln(move_count) / 2.5` formula vs Java's move-count-based reduction
- **Reverse Futility Pruning**: Prunes positions where static eval is far above beta
- **Full Check Extensions**: Always extends by 1 ply when in check (vs Java's fractional approach)

### Move Ordering

| Stage | Java Rival | Rusty Rival |
|-------|-----------|-------------|
| 1. Hash Move | Yes (from TT) | Yes (from TT) |
| 2. Good Captures | SEE > 0 (score 110+) | SEE ordering |
| 3. Killer Moves | Score 105-106 | Score 900000+ |
| 4. History Moves | 90 + history score | Depth-weighted history |
| 5. Quiet Moves | 50 + PST/2 | PST-based |

**Java Rival has more sophisticated move scoring:**
- Mate killer moves (separate from regular killers)
- History score as success/fail ratio (0-10 scale)
- Bad captures with good history get special treatment

---

## Evaluation Comparison

### Material Values

Both engines use similar material values with game-phase interpolation.

### Evaluation Terms

| Term | Java Rival | Rusty Rival |
|------|-----------|-------------|
| Material | Yes | Yes |
| Piece-Square Tables | Yes (opening + endgame interpolation) | Yes (opening + endgame interpolation) |
| Pawn Structure | | |
| - Doubled Pawns | Yes | Yes |
| - Isolated Pawns | Yes | Yes |
| - Backward Pawns | Yes | Yes |
| - Passed Pawns | Yes (detailed) | Yes (rank-based bonus) |
| - Connected Passed Pawns | Yes | Yes |
| - Guarded Passed Pawns | Yes | Yes |
| King Safety | | |
| - Pawn Shield | Yes | Yes (pattern matching) |
| - King Threats | Yes (attack counting) | Yes (danger zone attacks) |
| - King Activity (endgame) | Yes | Yes |
| Piece Evaluation | | |
| - Bishop Pair | Yes | Yes (+ fewer pawns bonus) |
| - Bishop Mobility | Yes | Yes |
| - Knight Outposts | No | Yes |
| - Knight Forks | No | Yes (threat detection) |
| - Rook on Open File | Yes | Yes |
| - Rook on Semi-Open File | Yes | Yes |
| - Rook on 7th Rank | Yes | Yes |
| - Connected Rooks | Yes | Yes (on same file) |
| - Queen Mobility | Yes | Yes |
| Trade Bonuses | Yes | No |
| Threat Evaluation | Yes | Partial (king threats only) |
| Endgame Adjustments | Yes | Yes |

### Key Evaluation Differences

**Java Rival has:**
- **Trade Bonuses**: Encourages trading pieces when ahead, trading pawns when behind
- **Threat Evaluation**: Counts attacks on pieces (beyond just king safety)
- **More detailed king safety**: Considers attack squares, piece types attacking
- **Blocked piece detection**: Penalty for blocked bishops/knights

**Rusty Rival has:**
- **Knight Fork Threats**: Detects and scores potential knight forks on king + major pieces
- **Knight Outposts**: Bonuses for knights protected by pawns in enemy territory
- **Bishop Pair with Pawn Penalty**: More bonus for bishop pair with fewer pawns
- **Endgame Draw Detection**: KPK draws, wrong-colored bishop draws

### Draw Detection

| Draw Type | Java Rival | Rusty Rival |
|-----------|-----------|-------------|
| Insufficient Material | Yes | Yes |
| Same-Colored Bishops | Yes | Yes |
| KPK Opposition | No | Yes |
| Wrong-Colored Bishop | No | Yes |
| 50-Move Rule | Yes | Yes |
| Threefold Repetition | Yes | Yes |

---

## Why Java Rival Performs Well Despite Lower NPS

1. **Better Search Selectivity**: More aggressive pruning (threat extensions, delta pruning) means it searches more relevant lines

2. **Threat Extensions**: Extending when null move reveals threats helps find tactical shots

3. **Trade Bonuses**: Properly handling material imbalances improves practical play

4. **History-Based LMR**: Using history score to decide LMR means good moves found before are searched deeper

5. **Mate Killer Moves**: Special handling for moves that previously caused mate improves tactical play

6. **Fractional Extensions**: Accumulating partial extensions is more sophisticated than binary extend/don't-extend

---

## Potential Improvements for Rusty Rival

Based on this comparison, these features from Java Rival could help:

1. **Threat Extensions**: Extend search when null move reveals opponent threats
2. **Delta Pruning**: Skip hopeless captures in quiescence search
3. **Trade Bonuses**: Encourage favorable piece/pawn trades based on material balance
4. **Better History Integration**: Use history score to influence LMR decisions
5. **Mate Killers**: Track moves that caused mate separately from regular killers
6. **General Threat Evaluation**: Score attacks on pieces beyond just king safety

---

## File Locations

### Java Rival
- Search: `rivalchess-engine/src/main/java/com/netsensia/rivalchess/engine/search/Search.kt`
- Evaluation: `rivalchess-engine/src/main/java/com/netsensia/rivalchess/engine/eval/Evaluate.kt`
- Move Scoring: `rivalchess-engine/src/main/java/com/netsensia/rivalchess/engine/search/MoveScoreExtensions.kt`
- SEE: `rivalchess-engine/src/main/java/com/netsensia/rivalchess/engine/eval/see/StaticExchangeEvaluator.kt`

### Rusty Rival
- Search: `rusty-rival/src/search.rs`
- Evaluation: `rusty-rival/src/evaluate.rs`
- Move Scoring: `rusty-rival/src/move_scores.rs`
- SEE: `rusty-rival/src/see.rs`
