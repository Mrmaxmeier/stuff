#!/bin/bash
set -e

FILTER='[0-9]+\.[0-9]+(\.[0-9]+)?'
RUNNING=$(uname -r | rg -o "$FILTER")
INSTALLED=$(pacman -Qi linux | grep -i "version" | rg -o "$FILTER")

echo "  RUNNING: $RUNNING"
echo "INSTALLED: $INSTALLED"
