#!/bin/bash

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: e <input-file> <version-number>"
    echo "Example: e test1 11"
    exit 1
fi

dir=$(ls -d engines/v"$2"-* 2>/dev/null | head -1)

if [ -z "$dir" ]; then
    echo "No engine found for version $2"
    exit 1
fi

input_file="inputs/$1.txt"
if [ ! -f "$input_file" ]; then
    echo "Input file not found: $input_file"
    exit 1
fi

results_dir="inputs/results/$1"
mkdir -p "$results_dir"
output_file="$results_dir/v$2.txt"
"$dir/rusty-rival" < "$input_file" | tee "$output_file"
