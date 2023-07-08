#!/bin/bash

cd "$(dirname "$0")/.." || exit 1

cargo build -r --bin chain-solver
cargo build -r --bin tanakh-solver

dir="$(mktemp -d)"

cp target/release/chain-solver "${dir}/chain-solver" || exit 1
cp target/release/tanakh-solver "${dir}/tanakh-solver" || exit 1

for i in 8; do
    "${dir}/chain-solver" "$i" -o "${dir}/$i.json"

    "${dir}/tanakh-solver" "$i" --initial-solution "${dir}/$i.json" --threads=1 --time-limit=60
done
