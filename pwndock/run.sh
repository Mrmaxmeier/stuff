docker run --rm --mount type=bind,source=(pwd),destination=/cwd -p 2222:22 -p 1337:1337 --cap-add=SYS_PTRACE --security-opt seccomp=unconfined -it pwn16.04 /bin/bash

