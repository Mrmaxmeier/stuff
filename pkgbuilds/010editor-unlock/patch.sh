#!/usr/bin/env sh
SCRIPT="wa mov eax, 0xdb\; ret @  method.RRegister.CheckStatus_int__int_"
r2 -w -q -c "$SCRIPT" /usr/bin/010editor
