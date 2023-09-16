#!/bin/bash

set -euxo pipefail

sudo apt-get update && sudo apt-get install -y  rsync php php8.1-cli php

# install sysbench
if ! which sysbench > /dev/null ; then
    curl -s https://packagecloud.io/install/repositories/akopytov/sysbench/script.deb.sh | sudo bash
    sudo apt-get -y install sysbench
fi

# install phoronix-test-suite
if ! which phoronix-test-suite > /dev/null ; then
    wget http://phoronix-test-suite.com/releases/repo/pts.debian/files/phoronix-test-suite_10.8.4_all.deb
    sudo dpkg -i phoronix-test-suite_10.8.4_all.deb
    sudo apt-get install -f
    sudo apt --fix-broken install
fi
