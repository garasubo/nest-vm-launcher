#!/bin/bash

set -euxo pipefail

phoronix-test-suite batch-run build-linux-kernel
