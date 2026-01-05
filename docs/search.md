# Search Algorithm Documentation

This document describes the search algorithm implemented in Rusty Rival, including all enhancements and optimizations currently in use.

## Overview

Rusty Rival uses an **iterative deepening alpha-beta search** with principal variation search (PVS). The search is structured as follows:

```
iterative_deepening()
  └── start_search()           # Root move iteration
        └── search()           # Main alpha-beta with extensions/reductions
              ├── quiesce()    # Quiescence search at depth 0
              ├── null move    # Forward pruning
              └── search()     # Recursive calls
```

## Core Algorithm

### Iterative Deepening

The engine searches progressively deeper from depth 1 up to the maximum depth. Key features:

- **Aspiration Windows**: Starting from depth 2+, the search uses narrow windows around the previous score
  - Initial window: previous score ± 25 centipawns
  - Widening sequence: [25, 50, 100, 200, 400, 800] then full window
  - Re-searches when score falls outside the window

- **Move Ordering at Root**: Moves are sorted by score from previous iteration for better alpha-beta cutoffs

- **Time Management**: Checks time every 1000 nodes; sends info every 1M nodes

### Alpha-Beta Search

Standard fail-soft alpha-beta with negamax scoring. The search function includes:

1. **Early terminations**:
   - Stop flag check
   - Draw detection (repetition, 50-move rule, insufficient material)
   - Tablebase probe (when ≤6 pieces)

2. **Transposition table lookup**:
   - 128MB hash table (~6M entries)
   - Stores: score, depth, bound type (exact/upper/lower), best move, zobrist lock
   - Mate score adjustment for correct ply distance

3. **Forward pruning** (see below)

4. **Move generation and iteration**

5. **Recursive search with extensions/reductions**

6. **Transposition table storage**

## Search Enhancements

### 1. Transposition Table (Hash Table)

**Location**: `search.rs:344-386`, `store_hash_entry()` at line 226

- **Size**: 128MB with ~6M entries (22 bytes per entry)
- **Replacement**: Depth-preferred with version-based aging
- **Contents**: Score, best move, depth, bound type, zobrist lock
- **Mate Score Handling**: Adjusted relative to current ply when storing/retrieving

### 2. Null Move Pruning

**Location**: `search.rs:410-432`

Skips the opponent's turn to see if position is still good, allowing deep branches to be pruned.

- **Minimum depth**: 4 ply
- **Reduction formula**: `depth - 1 - (3 + depth/6)`
  - At depth 4: reduces by 3 (searches at depth 0)
  - At depth 6: reduces by 4
  - At depth 12: reduces by 5
- **Conditions for use**:
  - Not already on a null move (avoids recursive null moves)
  - Scouting (zero-width window)
  - Not in check
  - At least 2 non-pawn pieces on the board (both sides combined)
- **Cutoff**: If null move search returns ≥ beta, return beta

### 3. Late Move Reductions (LMR)

**Location**: `search.rs:564-575`, `lmr_scout_search()` at line 685

Reduces search depth for moves that are unlikely to be best.

- **Minimum depth**: 3 ply
- **Legal moves before attempting**: 4 (first 4 moves searched fully)
- **Reduction amount**: 2 ply
- **Conditions**:
  - No extensions applied
  - Move is not a capture or promotion (non-tactical)
  - Move is not a killer move at current ply
  - Move doesn't give check

**Re-search logic**:
1. Scout search with reduced depth
2. If score > alpha and LMR was applied: re-search at full depth with reduced window
3. If still > alpha: re-search at full depth with full window

### 4. Check Extensions

**Location**: `search.rs:436-438`, `extend()` function at line 276

- **Condition**: Position is in check
- **Extension**: 1 ply
- **Limit**: Only one extension per path to avoid explosion
- **Depth limit**: Only extends if `ply < iterative_depth * 2`

### 5. Internal Iterative Deepening (IID)

**Location**: `search.rs:440-444`

When no hash move is available at high depth, does a shallow search to find a good move to try first.

- **Minimum depth for IID**: 3 ply
- **IID search depth**: Reduces by 2 ply from current depth
- **Condition**: Not scouting (full window) and no hash move verified

### 6. Reverse Futility Pruning (Beta Pruning)

**Location**: `search.rs:392-398`

At shallow depths, if static eval is much better than beta, prune the branch.

- **Maximum depth**: 3 ply
- **Margin**: 200 centipawns per depth (200/400/600)
- **Conditions**: Scouting, not in check, beta not near mate
- **Returns**: `lazy_eval - margin` (fail soft)

### 7. Futility Pruning (Alpha Pruning)

**Location**: `search.rs:400-408`, move loop at 559-561

Two-phase pruning for hopeless positions:

**Phase 1 - Flag Setting**:
- Depth 1-8 with margins: [128, 192, 256, 320, 384, 448, 512, 576]
- Conditions: Scouting, not in check, alpha not near mate
- Sets `alpha_prune_flag` if `eval + margin < alpha`

**Phase 2 - Move Pruning**:
- Skips non-tactical moves (not captures/promotions)
- Only after first legal move found
- Only if move doesn't give check

### 8. Principal Variation Search (PVS)

**Location**: `search.rs:577-581`

After finding a move that improves alpha:
1. Search remaining moves with scout (zero-width) window first
2. If scout fails high, re-search with full window

### 9. Staged Move Generation

**Location**: `search.rs:491-544`

Generates moves in phases to maximize cutoffs before generating all moves:

1. **Hash move first** (tried before any generation)
2. **Check evasions** when in check (all evasions generated together)
3. **Captures** generated first (when not in check)
4. **Quiet moves** generated only if needed

### 10. Move Ordering

**Location**: `move_scores.rs:72-132`, `score_move()`

Moves are scored for ordering:

| Priority | Score | Move Type |
|----------|-------|-----------|
| 1 | 3000 + SEE | Good captures (positive SEE) |
| 2 | 3000 + queen value | Queen promotions |
| 3 | 3000 + pawn value | En passant captures |
| 4 | 1000 | Mate killer from current ply |
| 5 | 750 | Killer move 1 (current ply) |
| 6 | 400 | Killer move 2 (current ply) |
| 7 | 300 | Killer move 1 (ply - 2) |
| 8 | 200 | Killer move 2 (ply - 2) |
| 9 | 0-500 | History heuristic (scaled) |
| 10 | 250 | Pawn push to 7th/2nd rank |
| 11 | 50 | Passed pawn push to 6th/3rd rank |
| 12 | 3/2/1 | Under-promotions (rook/bishop/knight) |

**Pick function**: Uses linear scan with swap-remove (`pick_high_score_move`) rather than full sort.

### 11. Killer Move Heuristic

**Location**: `search.rs:821-829`, `update_killers()`

- **Killers per ply**: 2
- **Mate killer**: Separate slot for moves that caused mate cutoffs
- **Update policy**: New killer pushes old one to slot 2; captures and promotions excluded

### 12. History Heuristic

**Location**: `search.rs:771-802`, `update_history()`

Tracks which quiet moves cause cutoffs:

- **Table structure**: [12 pieces][64 from-squares][64 to-squares] = 49,152 entries
- **Positive update**: `depth²` when move causes beta cutoff
- **Negative update**: `-depth * 1` or `-depth * 2` (more penalty if score < alpha)
- **Scaling**: History scores scaled 0-500 for move ordering
- **Overflow protection**: All scores halved when max exceeds i64::MAX/2

### 13. Static Exchange Evaluation (SEE)

**Location**: `see.rs`

Used for:
1. **Capture ordering**: Added to base capture score (3000)
2. **Quiescence pruning**: Only search captures with SEE > 0

**Implementation**: Recursive negamax-style evaluation of capture sequences on a single square.

## Quiescence Search

**Location**: `quiesce.rs`

Called when main search reaches depth 0 to resolve tactical instability.

- **Maximum depth**: 100 ply
- **Stand-pat**: Can return current eval if it beats beta
- **Move generation**: Captures only (including en passant and queen promotions)
- **Pruning**: Uses SEE to skip losing captures (SEE ≤ 0)
- **Move ordering**: By MVV-LVA (victim value - attacker bonus)

## Tablebase Integration

**Location**: `search.rs:316-325`

- **Probing**: WDL (Win/Draw/Loss) at every node when ≤6 pieces
- **DTZ probing**: Used in root for reporting progress
- **Score mapping**: Win/Loss converted to mate-like scores

---

## Areas for Potential Improvement

Based on analysis of the current implementation and common chess programming techniques, here are areas worth investigating:

### High Priority

#### 1. Singular Extensions

**What it is**: When one move appears significantly better than all alternatives (from TT), extend its search depth.

**Why it helps**: Avoids missing tactics where only one good move exists. Most strong engines use this. Currently we only extend for check.

**Implementation complexity**: Medium. Need to do a verification search at reduced depth excluding the hash move.

#### 2. More Aggressive LMR

**Current state**: Fixed 2-ply reduction after first 4 moves.

**Improvement**: Use a logarithmic formula based on depth AND move number:
```
reduction = floor(0.5 + ln(depth) * ln(moveNumber) / 2.0)
```

This allows much deeper reductions for late moves at high depths (3-4+ ply reductions), dramatically increasing search speed.

#### 3. Countermove Heuristic

**What it is**: Track which move refuted each opponent move, and boost that countermove in ordering.

**Why it helps**: Complements killers for positional play. If opponent played Nf3, track which of our moves historically refuted it.

**Implementation**: Simple 2D array [piece][to-square] -> Move. Very low overhead.

### Medium Priority

#### 4. Late Move Pruning (LMP)

**What it is**: At low depths, completely skip late quiet moves (don't even search them).

**Why it helps**: More aggressive than LMR - doesn't waste any time on moves unlikely to matter.

**Typical thresholds**:
- Depth 1: skip after 5 moves
- Depth 2: skip after 8 moves
- Depth 3: skip after 12 moves

#### 5. History Pruning

**What it is**: Prune moves with very negative history scores.

**Why it helps**: History tracks long-term move quality. If a move consistently fails, it's safe to prune at low depths.

#### 6. Improving/Worsening Detection

**What it is**: Track if eval is improving from grandparent node (ply - 2).

**Why it helps**:
- Improving positions: be more conservative with pruning
- Worsening positions: can prune more aggressively

Used by Stockfish for null move, futility, and LMR tuning.

#### 7. Better Move Picker (Staged Selection)

**Current state**: Linear scan through all scored moves.

**Improvement**: Generate moves lazily in stages:
1. Try hash move (no generation)
2. Generate captures, pick best one by one
3. Generate killer moves
4. Generate quiet moves, pick best one by one

This avoids scoring ALL moves upfront when a cutoff happens early.

### Lower Priority (But Worth Considering)

#### 8. Probcut

**What it is**: At high depth, do a shallow search with raised beta. If it fails high, the position is probably winning and can be cut.

**Why it helps**: Additional forward pruning at internal nodes.

#### 9. Multi-Cut

**What it is**: At high depth, if multiple moves fail high at shallow depth, assume the position is good.

**Why it helps**: Additional pruning when position has many good moves.

#### 10. Razoring

**What it is**: At low depths, if eval is very far below alpha, drop into qsearch directly.

**Current state**: We have alpha pruning which is similar but per-move. Razoring would skip the entire subtree.

#### 11. SEE Pruning in Main Search

**Current state**: SEE only used in qsearch.

**Improvement**: Use SEE to prune bad captures in main search too (not just late quiet moves).

### Performance/Structural

#### 12. Prefetch Hash Table

**What it is**: When making a move, prefetch the hash entry for the child position.

**Why it helps**: Hides memory latency. Hash table is large and often cache-misses.

#### 13. Tune Parameters

Many constants could be tuned via SPSA or similar:
- LMR thresholds and reduction amounts
- Futility margins
- Null move reduction formula
- History scaling factors
- Aspiration window sizes

### What NOT to Change

Some things are already well-implemented:

- **Make/Unmake pattern**: Already using in-place moves with UnmakeInfo
- **Transposition table**: Good size and replacement scheme
- **Killer moves**: Standard 2-killer implementation works well
- **Check extensions**: Simple and effective
- **SEE**: Full recursive implementation (some engines use approximations)

---

## Recommended Next Steps

1. **Singular Extensions** - High impact, used by all top engines
2. **Logarithmic LMR** - Easy change, potentially large speedup
3. **Countermove Heuristic** - Simple addition to move ordering
4. **Late Move Pruning** - Complements LMR well

These four improvements together could provide significant Elo gains while being relatively straightforward to implement and test.
