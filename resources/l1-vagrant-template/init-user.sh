#!/bin/bash

set -euxo pipefail

mkdir -p ~/.ssh
vagrant plugin install vagrant-libvirt

pushd /home/vagrant/l2-vagrant
vagrant up
vagrant ssh-config > ~/.ssh/config
ssh vagrant@l2-vagrant "echo 'hello world'"
popd
