#!/bin/bash

cd "$(dirname "$0")/.." || exit 1

dir="$(mktemp -d)"

hs="hungarian-solver"
cargo build -r --bin $hs
cp target/release/$hs "${dir}/$hs" || exit 1

cs="chain-solver"
cargo build -r --bin $cs
cp target/release/$cs "${dir}/$cs" || exit 1

ds="dp-solver"
cargo build -r --bin $ds
cp target/release/$ds "${dir}/$ds" || exit 1

: "${FROM:=1}"
: "${TO:=55}"

for i in $(seq $FROM $TO); do
    echo "===== PROBLEM $i ====="

    # "${dir}/$hs" "$i" -o "${dir}/hs_$i.json" -s true
    # "${dir}/$hs" "$i" -o "${dir}/hs_gap_$i.json" -a gap -s true
    # "${dir}/$hs" "$i" -o "${dir}/hs_fetch_$i.json" -a fetch

    # "${dir}/$cs" "$i" -o "${dir}/cs_$i.json" -s true

    "${dir}/$ds" "$i" -o "${dir}/ds_$i.json" -s
done
