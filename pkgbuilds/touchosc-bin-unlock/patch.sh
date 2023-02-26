#!/usr/bin/env sh
SCRIPT="""\
f sym.std_string_construct_empty @ sym.std::string::_S_empty_rep__+0x10;\
f sym.poco_dynamic_parse_string @ sym.Poco::Dynamic::Var::parse_std::string_const_;\
waf /usr/share/touchosc-bin-unlock/IsUnlocked.asm @ sym.LXUnlockStatus::IsUnlocked___const\
"""
r2 -w -q -c "$SCRIPT" /usr/bin/TouchOSC
