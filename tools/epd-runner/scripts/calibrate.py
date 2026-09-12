#!/usr/bin/env python3
"""NET-1288 calibration: for every (suite, mode, budget) in the store, order the engines by
score and count violations of the known orderings. Known pairs (weaker <
stronger): the rusty-rival release series where a gain was measured in games
(59<60 +15, 60<61 +25, 61<62 +13, 62<63 +12; 63 and 64 are level), the capped
Stockfish rungs (2600<2800<3000<full), every peer above rusty-rival 1.0.64,
and full Stockfish above everything. A pair counts as a violation when the
weaker engine scores strictly higher; a tie is neither."""
import json, glob, sys, collections
root = sys.argv[1] if len(sys.argv) > 1 else 'epd/results'
files = [json.load(open(f)) for f in glob.glob(f'{root}/*/*.json')]
def key(e):
    return f"{e['family']}:{e['label']}"
rival = [f"rusty-rival:{v}" for v in ["1.0.59","1.0.60","1.0.61","1.0.62","1.0.63","1.0.64"]]
sf = ["stockfish:sf-2600","stockfish:sf-2800","stockfish:sf-3000","stockfish:stockfish"]
peers = ["ethereal:ethereal","berserk:berserk","stash:stash","obsidian:obsidian"]
pairs = [(rival[i], rival[i+1]) for i in range(4)]            # 59<60<61<62<63
pairs += [(sf[i], sf[i+1]) for i in range(3)]
pairs += [("rusty-rival:1.0.64", p) for p in peers + sf[1:]]  # peers and sf-2800+ above rival
pairs += [(p, "stockfish:stockfish") for p in peers + sf[:3]]
pairs += [("stockfish:sf-2600", "rusty-rival:1.0.64")]         # rival ~3100 beats sf-2600 98%
cells = collections.defaultdict(dict)   # (suite, mode, budget, threads) -> engine -> percent
for f in files:
    e = key(f['engine'])
    for r in f['runs']:
        s = r['summary']
        pct = 100.0 * s['points'] / s['max_points'] if s.get('points') is not None and s.get('max_points') else (100.0 * s['solved'] / s['total'] if s['total'] else 0)
        cells[(r['suite']['name'], r['mode'], r['budget'], r['threads'], r.get('concurrency', 1))][e] = round(pct, 2)
rows = []
for k in sorted(cells):
    sc = cells[k]
    tested = [(a, b) for a, b in pairs if a in sc and b in sc]
    if len(tested) < 6:
        continue
    viol = [(a, b) for a, b in tested if sc[a] > sc[b]]
    ties = [(a, b) for a, b in tested if sc[a] == sc[b]]
    rows.append((k, len(tested), len(viol), len(ties), viol, sc))
print(f"{'suite':<14}{'mode':<7}{'budget':>9}{'thr':>4}{'conc':>5}  pairs  viol  ties  worst violations")
for (suite, mode, budget, thr, conc), n, v, t, viol, sc in rows:
    worst = ", ".join(f"{a.split(':')[1]}({sc[a]})>{b.split(':')[1]}({sc[b]})" for a, b in viol[:4])
    print(f"{suite:<14}{mode:<7}{budget:>9}{thr:>4}{conc:>5}  {n:>5}  {v:>4}  {t:>4}  {worst}")
print()
print("Rival series by (suite, budget): percent for 1.0.59 .. 1.0.64")
for (suite, mode, budget, thr, conc), n, v, t, viol, sc in rows:
    print(f"  {suite:<14}{mode:<7}{budget:>9}  " + "  ".join(f"{sc.get(r, float('nan')):6.1f}" for r in rival) + f"   peers: " + " ".join(f"{p.split(':')[1][:4]}={sc.get(p, float('nan')):.1f}" for p in peers + sf))
