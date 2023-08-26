#!/bin/bash

set -euxo pipefail

vagrant plugin install vagrant-libvirt

pushd /home/vagrant/l2-vagrant
vagrant up
popd