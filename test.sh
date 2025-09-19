#!/bin/sh

set -eux

for lang in c++ python; do
    make -C examples/multi-language/$lang clean && make -C examples/multi-language/$lang run
done
