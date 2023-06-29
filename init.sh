#!/bin/bash

cd "$(dirname $0)"

for f in hooks/*; do
    ln -sf "../../$f" "./.git/hooks/$(basename $f)"
done
