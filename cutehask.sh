FILENAME=rusty-rival-$1-v-rival-haskell-exe
ENGINEDIR=/home/chris/ChessEngines
LOGDIR=$ENGINEDIR/logs
BOOKDIR=$ENGINEDIR/books

cutechess-cli -engine cmd="$ENGINEDIR/rusty-rival-$1" -engine cmd="$ENGINEDIR/rival-haskell-exe" -each proto=uci book="$BOOKDIR/ProDeo.bin" timemargin=1500 st=0.25 nodes=100000000 -resign movecount=10 score=600 -rounds $2 -pgnout $LOGDIR/$FILENAME.pgn -epdout $LOGDIR/$FILENAME.epd $3
