if [ $# -ne 1 ];
    then echo "Only one argument expected"
    exit
fi     
sed -i "s/Rusty Rival |.*|/Rusty Rival |$1|/g" src/uci.rs
cargo build --release
cp target/release/rusty-rival ~/ChessEngines/rusty-rival-$1
git add -A
git commit -m "Tagged $1"
git tag $1
git push



