push rax
push rbx

mov rbx, rdi

xor rax, rax
push rax
push rax
push rax

mov rdi, rsp

; f sym.std_string_construct_empty @ sym.std::string::_S_empty_rep__+0x10
call sym.std_string_construct_empty

push 0x65757274 ; "true"
mov rsi, rsp

push rbx
push rdi
call sym.std::string::assign_char_const_
pop rsi
pop rdi
; f sym.poco_dynamic_parse_string @ sym.Poco::Dynamic::Var::parse_std::string_const_
call sym.poco_dynamic_parse_string

pop rax
pop rax
pop rax
pop rax

pop rbx
pop rax
ret

