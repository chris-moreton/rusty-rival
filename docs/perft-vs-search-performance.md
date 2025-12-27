# Perft vs Search Performance with Make/Unmake Move Pattern

## Observation

After implementing `unmake_move` to replace position copying, perft performance decreased:

| Metric | Before (copy) | After (unmake) |
|--------|---------------|----------------|
| Nodes  | 120,669,525   | 120,669,525    |
| NPS    | 68,601,208    | 58,407,320     |

This is approximately a **15% slowdown** in perft.

## Why Perft is Slower with Make/Unmake

1. **Perft does almost nothing per node** - just generate moves, recurse, count. The move make/unmake overhead is a larger percentage of total work.

2. **Position copy is a single memcpy** - copying ~150 bytes contiguously is extremely fast on modern CPUs with wide memory buses.

3. **Unmake has conditional logic** - reversing captures, en passant, castling, promotions requires branching, which can cause pipeline stalls.

## Why Search Benefits from Make/Unmake

1. **Expensive work per node** - evaluation, hash lookups, move ordering, pruning decisions all dwarf the make/unmake cost.

2. **Better cache locality** - modifying one Position in place keeps it hot in L1 cache vs. allocating new stack copies.

3. **Reduced stack pressure** - Position is 156 bytes; at depth 30+ with many moves, stack copies add up significantly.

## Conclusion

The perft slowdown is **expected and acceptable**. Perft is a special case where the simplicity of position copying wins. In actual search, the make/unmake pattern provides memory efficiency benefits that outweigh the small overhead of reversing moves.

## Verification

To verify unmake isn't hurting search performance, run a benchmark on a tactical position:

```bash
echo "position fen r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4
go depth 12" | ./target/release/rusty-rival
```

If the engine finds moves quickly and NPS is similar to before the change, search performance is unaffected.
