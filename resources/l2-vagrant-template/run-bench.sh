#!/bin/bash

set -euxo pipefail

sysbench --test=cpu run

