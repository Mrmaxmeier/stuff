FROM ubuntu:19.10

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update && \
	apt-get install -y gdb vim tmux git python3 python3-pip socat wget htop ltrace strace patchelf && \
	apt-get install -y build-essential libssl-dev libffi-dev python-dev && \
	wget -q -O- https://github.com/hugsy/gef/raw/master/scripts/gef.sh | sh

RUN apt-get install -y keystone && \
	pip3 install keystone

RUN cd /tmp/ && \
	git clone https://github.com/Gallopsled/pwntools && \
	cd pwntools/ && \
	pip3 install --user --editable .

RUN wget https://raw.githubusercontent.com/hellman/fixenv/master/r.sh -O /usr/bin/fixenv && \
	chmod +x /usr/bin/fixenv

RUN dpkg --add-architecture i386 && \
	apt-get update && \
	apt-get install -y libc6:i386 libstdc++6:i386 && \
	apt-get install -y libc6-dbg:i386 libncurses5:i386 libseccomp-dev:i386

RUN apt-get -y install openssh-server && \
	sed -i -e 's/#AuthorizedKeysFile/AuthorizedKeysFile/g' /etc/ssh/sshd_config && \
	sed -i -e 's/PermitRootLogin no/PermitRootLogin yes/g' /etc/ssh/sshd_config && \
	mkdir /root/.ssh && \
	echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICzgRxdNS8tb4HvZp0UDmhky12MZXR0srwPQ3iRNaJRT flo" > /root/.ssh/authorized_keys && \
	chmod -R og-rwx /root/.ssh && \
	service ssh start

COPY ./ynetd /usr/bin/


# apt-get -y install locales
# locale-gen en_US.UTF-8

ENV PYTHONIOENCODING=utf8
ENV LC_CTYPE=C.UTF-8
