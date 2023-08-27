#!/bin/bash

set -euxo pipefail

ssh l2-vagrant "./run-bench.sh > /tmp/bench-results.txt"
rsync -avr l2-vagrant:/tmp/bench-results.txt bench-results.txt
