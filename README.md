# Rusty Rival

Chess move generation routines written in Rust.

## Running Perft

The only useful command at the moment is *perft*, to determine the total number
of positions encountered while playing through every move and every response to a certain depth.

Although the move-generation and move-making routines are solid and accurate, the 
command line interface is currently hacked together, so try to avoid typos.

```
cargo run --release

position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1
READY
go perft 7
a5a4: 14,139,786  68,974,565 nps
a5a6: 16,022,983  74,845,580 nps
b4b1: 19,481,757  77,087,773 nps
b4b2: 12,755,330  77,515,349 nps
b4b3: 15,482,610  77,882,466 nps
b4f4: 3,069,955  78,064,051 nps
b4e4: 14,187,097  78,047,184 nps
b4d4: 15,996,777  78,099,996 nps
b4c4: 17,400,108  78,280,391 nps
b4a4: 11,996,400  78,291,255 nps
g2g3: 4,190,119  78,313,269 nps
g2g4: 13,629,805  78,198,877 nps
e2e3: 11,427,551  78,095,804 nps
e2e4: 8,853,383  78,005,965 nps
Time elapsed in perft is: 2.290587309s
178633661 nodes 78005965.50218341 nps
```




