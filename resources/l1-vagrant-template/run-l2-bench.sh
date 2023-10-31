#!/bin/bash

set -euxo pipefail

ssh l2-vagrant "./run-bench.sh | tee /tmp/bench-results.txt 2&>1 | tee ~/run-bench.log"
rsync -avr l2-vagrant:/tmp/bench-results.txt bench-results.txt
