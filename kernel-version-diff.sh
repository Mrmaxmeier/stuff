#!/bin/bash
set -e

RUNNING=$(uname -r | sed 's/-ARCH//')
INSTALLED=$(pacman -Qi linux | grep -i "version" | sed 's/.*: //')

echo "  RUNNING: $RUNNING"
echo "INSTALLED: $INSTALLED"
