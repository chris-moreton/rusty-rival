#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: e <version-number> [input-file]"
    echo "Example: e 1, e 7, e 10 test1"
    exit 1
fi

dir=$(ls -d engines/v"$1"-* 2>/dev/null | head -1)

if [ -z "$dir" ]; then
    echo "No engine found for version $1"
    exit 1
fi

if [ -n "$2" ]; then
    input_file="inputs/$2.txt"
    if [ ! -f "$input_file" ]; then
        echo "Input file not found: $input_file"
        exit 1
    fi
    results_dir="inputs/results/$2"
    mkdir -p "$results_dir"
    output_file="$results_dir/v$1.txt"
    "$dir/rusty-rival" < "$input_file" | tee "$output_file"
else
    exec "$dir/rusty-rival"
fi
