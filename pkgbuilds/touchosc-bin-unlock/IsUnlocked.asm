push rbx

mov rbx, rdi

xor rax, rax
push rax
push rax
push rax

mov rdi, rsp

nop
call sym.std::__cxx11::basic_string_char__std::char_traits_char___std::allocator_char___::basic_string_std::allocator_char__const_

push 0x65757274 ; "true"
mov rsi, rsp

push rdi
call sym.std_string_assign_char_const
pop rsi

; TODO: r2's assembler fails to parse/find the symbol because it contains upper case chars .__.
mov rdi, 0
call sym.poco_dynamic_parse_string
mov rbx, rax

pop rax
pop rax
pop rax
pop rax

mov rax, rbx
pop rbx
ret

