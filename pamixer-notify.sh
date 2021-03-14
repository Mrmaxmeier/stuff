#!/bin/bash

notify() {
    notify-send -t 1000 --hint=int:transient:1 "$1" "Current is $(pamixer --get-volume-human)" --icon=multimedia-volume-control
}

case "$1" in
  up)
    pamixer -i 2
    notify "Volume +2%"
    ;;
  down)
    pamixer -d 2
    notify "Volume -2%"
    ;;
  mute)
    pamixer -t
    notify "Toggle mute"
    ;;
  *)
    echo "wat"
    exit 1
    ;;
esac
exit 0
