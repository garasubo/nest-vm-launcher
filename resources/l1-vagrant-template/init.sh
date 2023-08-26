#!/bin/bash

set -euxo pipefail

echo "hello world"

if [ ! -e "/usr/share/keyrings/hashicorp-archive-keyring.gpg" ]; then
    wget -O- https://apt.releases.hashicorp.com/gpg | sudo gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg
fi
if [ ! -e "/etc/apt/sources.list.d/hashicorp.list" ]; then
    echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list
fi
sudo apt-get update && sudo apt-get install -y qemu libvirt-daemon-system libvirt-dev ebtables libguestfs-tools vagrant ruby-fog-libvirt build-essential

sudo adduser vagrant libvirt
