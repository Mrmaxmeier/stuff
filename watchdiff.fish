#!/bin/env fish
set path $argv[1]
if test ! -e "$path"
    echo "file not found" $path
    exit
end

set copy (mktemp)
set --erase argv[1]
trap "rm $copy" EXIT INT

while true
	if diff "$path" "$copy" > /dev/null
		sleep 1
	else
		echo "diff $path $copy $argv"
		diff "$path" "$copy" $argv
		cp "$path" "$copy"
		sleep 5
	end
end
