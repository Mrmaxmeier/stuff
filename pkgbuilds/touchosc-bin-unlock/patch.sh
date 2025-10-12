#!/usr/bin/env sh
SCRIPT="""\
f sym.std_string_construct_empty @ sym.std::__cxx11::basic_string_char__std::char_traits_char___std::allocator_char___::basic_string_std::allocator_char__const_;\
f sym.std_string_assign_char_const @ sym.std::__cxx11::basic_string_char__std::char_traits_char___std::allocator_char___::assign_char_const_;\
f sym.poco_dynamic_parse_string @ sym.Poco::Dynamic::Var::parse_std::__cxx11::basic_string_char__std::char_traits_char___std::allocator_char____const_;\
waf /usr/share/touchosc-bin-unlock/IsUnlocked.asm @ sym.LXUnlockStatus::IsUnlocked___const\
"""
r2 -w -q -c "$SCRIPT" /usr/bin/TouchOSC
