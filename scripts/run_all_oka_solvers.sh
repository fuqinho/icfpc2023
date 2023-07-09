#!/bin/bash

cd "$(dirname "$0")/.." || exit 1

dir="$(mktemp -d)"

hs="hungarian-solver"

cargo build -r --bin $hs

cs="chain-solver"

cargo build -r --bin $cs

cp target/release/$hs "${dir}/$hs" || exit 1
cp target/release/$cs "${dir}/$cs" || exit 1

for i in {1..90}; do
    echo "===== PROBLEM $i ====="

    "${dir}/$hs" "$i" -o "${dir}/hs_$i.json"
    "${dir}/$hs" "$i" -o "${dir}/hs_gap_$i.json" -a gap

    "${dir}/$cs" "$i" -o "${dir}/cs_$i.json"
done
