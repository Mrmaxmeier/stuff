#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pyinfra>=3,<4"]
# ///
"""Idempotently provision a HyperQueue (hq) cluster with pyinfra.

    ./hq-deploy.py HOST... --primary HOST [--worker-on-primary] [--comfy] [--dry] [-- PYINFRA_ARGS]

Hosts are ssh-config aliases. The --primary host (deployed even if not listed
among HOSTs) runs `hq server` and exports /mnt/hq over NFS to its subnet;
all other hosts mount it there and run
`hq worker`. Both are systemd user units (`systemctl --user status hq-server`,
`journalctl --user -u hq-worker`), kept running via lingering; the server
persists its state in ~/.hq-journal. The server IP and subnet come from the
primary's default-route interface (override with --server-ip).

The primary can ssh to every worker by its alias (`ssh WORKER`) with a
dedicated key, generated on this machine under SSH_KEY_DIR/PRIMARY and only
accepted from the server IP. Users and ports come from this machine's
`ssh -G`, addresses from the worker's interface in the primary's subnet, and
host keys are read from the workers. Each worker gets its own file under
~/.ssh/hq.d on the primary, so deploying a subset of hosts doesn't drop the
others.

--comfy also installs btop/htop, configures tmux (mouse, colours) and sets the
timezone (--timezone, defaulting to this machine's).

Remote steps use sudo, so the ssh user needs passwordless sudo.
"""

from __future__ import annotations

import functools
import io
import ipaddress
import os
import re
import shlex
import subprocess
import sys

# Pin hq's server dir to the NFS share instead of relying on its ~/.hq-server
# default, which on workers only resolves through the symlink below.
HQ_SERVER_DIR_VAR = "HQ_SERVER_DIR"
HQ_DIR = "/mnt/hq"  # same path on every host: exported by the primary, mounted elsewhere
HQ = f"{HQ_DIR}/hq"
HQ_SERVER_DIR = f"{HQ_DIR}/.hq-server"
MAX_SESSIONS = 64
NFSD_THREADS = 64
NOFILE_LIMITS = ["*  soft  nofile  65535", "*  hard  nofile  65535"]
TMPFS_SIZE_GIB = 300
TMPFS_TMP_OPTS = f"defaults,size={TMPFS_SIZE_GIB}G,nr_inodes=50M,mode=1777"
PANIC_SYSCTLS = {
    "kernel.hardlockup_panic": 1,
    "kernel.softlockup_panic": 1,
    "kernel.panic": 30,  # seconds until reboot after a panic
}

# Controller-side home of the primary->worker keypairs, one dir per primary.
SSH_KEY_DIR = os.path.join(
    os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state"), "hq-deploy")
SSH_KEY_NAME = "id_hq"  # on the primary, in ~/.ssh
SSH_HQ_DIR = "hq.d"  # on the primary, in ~/.ssh: per-worker config + known_hosts

COMFY_PACKAGES = ["btop", "htop", "tmux", "dfc", "ncdu"]
TMUX_CONF = [
    "set -g mouse on",
    "set -g extended-keys on",
    "set -g extended-keys-format csi-u",
    'set -g default-terminal "tmux-256color"',
    # config equivalent of `tmux -2`, plus truecolor for xterm-likes
    'set -as terminal-features ",*:256"',
    'set -as terminal-features ",xterm*:RGB"',
]


# --- controller-side discovery ----------------------------------------------

@functools.cache
def discover_primary(primary: str, server_ip: str | None) -> tuple[str, str]:
    """Return the primary's (server IP, subnet CIDR) via ssh.

    The server IP defaults to the source address of the primary's default
    route; the CIDR is the prefix of the interface holding that address.
    """
    out = subprocess.run(
        ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", primary,
         "ip -o -4 route get 1.1.1.1; ip -o -4 addr show scope global"],
        text=True, stdout=subprocess.PIPE, check=True,
    ).stdout
    if server_ip is None and (m := re.search(r"\bsrc (\S+)", out)):
        server_ip = m.group(1)
    for addr in re.findall(r"\binet (\S+)", out):
        iface = ipaddress.ip_interface(addr)
        if str(iface.ip) == server_ip:
            return server_ip, str(iface.network)
    raise SystemExit(f"could not determine server IP/subnet on {primary!r}; pass --server-ip")


@functools.cache
def cluster_key(primary: str) -> tuple[str, str]:
    """Return (private key path, public key line) of the primary's ssh key,
    generating it on first use. The comment tags it in authorized_keys."""
    key = os.path.join(SSH_KEY_DIR, primary, "id_ed25519")
    if not os.path.exists(key):
        os.makedirs(os.path.dirname(key), mode=0o700, exist_ok=True)
        subprocess.run(
            ["ssh-keygen", "-q", "-t", "ed25519", "-N", "",
             "-C", f"hq-deploy@{primary}", "-f", key],
            check=True,
        )
    with open(f"{key}.pub") as f:
        return key, f.read().strip()


@functools.cache
def ssh_config(alias: str) -> dict[str, str]:
    """This machine's effective ssh options for `alias` (`ssh -G`)."""
    out = subprocess.run(
        ["ssh", "-G", alias], text=True, stdout=subprocess.PIPE, check=True,
    ).stdout
    return dict(line.split(" ", 1) for line in out.splitlines() if " " in line)


def local_timezone() -> str | None:
    """IANA name of this machine's timezone, from the /etc/localtime symlink."""
    _, sep, name = os.path.realpath("/etc/localtime").partition("zoneinfo/")
    return name if sep else None


# --- the pyinfra deploy -----------------------------------------------------

def build_operations() -> None:
    """Collected by pyinfra once per host; dispatches on the host's role."""
    from pyinfra import host, inventory
    from pyinfra.facts.server import Home

    def flag(key: str) -> bool:
        return str(host.data.get(key) or "").lower() in ("1", "true", "yes")

    primary = host.data.get("primary")
    if not primary:
        raise SystemExit("--primary (--data primary=...) is required")
    server_ip, cidr = discover_primary(primary, host.data.get("server_ip"))
    home = host.get_fact(Home)
    is_primary = host.name == primary

    packages = ["nfs-kernel-server" if is_primary else "nfs-common", "podman", "uidmap"]
    if flag("comfy"):
        packages += COMFY_PACKAGES
    install_packages(list(dict.fromkeys(packages)))
    if flag("comfy"):
        setup_comfy(host.data.get("timezone"))
    setup_user_systemd()

    if is_primary:
        deploy_server(home, server_ip, cidr, host.data.get("hq_url"))
        if flag("worker_on_primary"):
            deploy_local_worker(home)
        workers = [h for h in inventory if h.name != primary]
        setup_primary_ssh(home, cluster_key(primary)[0], workers, cidr)
    else:
        deploy_worker(home, f"{server_ip}:{HQ_DIR}")
        authorize_primary(home, cluster_key(primary)[1], server_ip)


def install_packages(packages: list[str]) -> None:
    from pyinfra.operations import apt

    apt.packages(
        name=f"apt: {', '.join(packages)}", packages=packages,
        update=True, cache_time=24 * 3600, _sudo=True,
    )


def setup_comfy(timezone: str | None) -> None:
    from pyinfra.operations import files, server

    for line in TMUX_CONF:
        files.line(
            name=f"tmux.conf: {line}", path="/etc/tmux.conf",
            line=line, escape_regex_characters=True, _sudo=True,
        )
    if timezone:
        server.timezone(name=f"timezone {timezone}", timezone=timezone, _sudo=True)


def setup_hq_env(home: str) -> None:
    """hq on PATH, HQ_SERVER_DIR in .bashrc, ~/.hq-server symlink."""
    from pyinfra.operations import files

    files.line(
        name="hq on PATH (.bashrc)", path=f"{home}/.bashrc",
        line=f'export PATH="{HQ_DIR}:$PATH"', escape_regex_characters=True,
    )
    files.line(
        name="HQ_SERVER_DIR (.bashrc)", path=f"{home}/.bashrc",
        line=rf"^export {HQ_SERVER_DIR_VAR}=",
        replace=f'export {HQ_SERVER_DIR_VAR}="{HQ_SERVER_DIR}"',
    )
    # hq's default server dir, for shells without HQ_SERVER_DIR (force:
    # replace a stale real directory there).
    files.link(
        name="~/.hq-server symlink", path=f"{home}/.hq-server",
        target=HQ_SERVER_DIR, force=True, force_backup=False,
    )


def deploy_server(home: str, server_ip: str, cidr: str, hq_url: str | None) -> None:
    from pyinfra import host
    from pyinfra.facts.files import File
    from pyinfra.facts.server import User
    from pyinfra.operations import files, server, systemd

    files.directory(name=f"{HQ_DIR} dir", path=HQ_DIR, user=host.get_fact(User), _sudo=True)
    files.directory(name=f"{HQ_SERVER_DIR} dir", path=HQ_SERVER_DIR)

    # None: nothing at HQ (False would be e.g. a symlink to an existing binary)
    if host.get_fact(File, HQ) is None:
        if hq_url and hq_url.endswith((".tar.gz", ".tgz")):
            files.download(name="fetch hq tarball", src=hq_url, dest="/tmp/hq.tgz")
            server.shell(
                name="extract hq binary",
                commands=[f"tar -xzf /tmp/hq.tgz -C {HQ_DIR} hq && "
                          f"chmod +x {HQ}"],
            )
        elif hq_url:
            files.download(name="fetch hq binary", src=hq_url, dest=HQ, mode="755")
        else:
            print(f"WARNING [{host.name}]: no hq binary at {HQ} and no --hq-url",
                  file=sys.stderr)

    setup_hq_env(home)

    for lim in NOFILE_LIMITS:
        files.line(
            name=f"limits.conf: {lim}", path="/etc/security/limits.conf",
            line=lim, escape_regex_characters=True, _sudo=True,
        )

    files.line(
        name=f"sshd MaxSessions {MAX_SESSIONS}", path="/etc/ssh/sshd_config",
        line=r"^#?\s*MaxSessions\s+", replace=f"MaxSessions {MAX_SESSIONS}",
        _sudo=True,
    )
    server.shell(
        name="reload sshd if MaxSessions differs",
        commands=[
            f'cur=$(sshd -T 2>/dev/null | awk "/^maxsessions/ {{print \\$2}}"); '
            f'[ "$cur" = "{MAX_SESSIONS}" ] || '
            f'(sshd -t && (systemctl reload ssh || systemctl reload sshd))'
        ],
        _sudo=True,
    )

    files.line(
        name="/etc/exports entry", path="/etc/exports",
        line=rf"^{re.escape(HQ_DIR)}\s",
        # async: ack writes before they hit disk; a primary crash may lose
        # recently written data, which we accept for the throughput.
        replace=f"{HQ_DIR} {cidr}(rw,async,no_subtree_check)",
        _sudo=True,
    )
    server.shell(name="exportfs -ra", commands=["exportfs -ra"], _sudo=True)

    # nfs-utils >= 2.x reads nfs.conf.d; Debian additionally passes
    # RPCNFSDCOUNT (default 8) on rpc.nfsd's command line, which wins.
    files.put(
        name=f"nfs.conf.d: threads={NFSD_THREADS}",
        src=io.StringIO(f"[nfsd]\nthreads={NFSD_THREADS}\n"),
        dest="/etc/nfs.conf.d/hq.conf", create_remote_dir=True, _sudo=True,
    )
    files.line(
        name=f"RPCNFSDCOUNT={NFSD_THREADS}", path="/etc/default/nfs-kernel-server",
        line=r"^RPCNFSDCOUNT=", replace=f"RPCNFSDCOUNT={NFSD_THREADS}", _sudo=True,
    )
    systemd.service(
        name="nfs-server up", service="nfs-server",
        running=True, enabled=True, _sudo=True,
    )
    # Resize the running nfsd pool in place rather than restarting it.
    server.shell(
        name=f"nfsd threads = {NFSD_THREADS}",
        commands=[f'[ "$(cat /proc/fs/nfsd/threads)" = {NFSD_THREADS} ] || '
                  f'rpc.nfsd {NFSD_THREADS}'],
        _sudo=True,
    )

    install_service(home, "hq-server",
                    f"{HQ} server start --host={server_ip} --journal=%h/.hq-journal")


def setup_worker_host() -> None:
    """kvm access, tmpfs /tmp and kernel limits for running many rootless
    containers."""
    from pyinfra import host
    from pyinfra.facts.hardware import Cpus
    from pyinfra.facts.server import Sysctl, User
    from pyinfra.operations import files, server

    user = host.get_fact(User)
    kvm = server.user(
        name="user in kvm group", user=user, groups=["kvm"], append=True, _sudo=True,
    )
    # The lingering user manager (and so hq-worker) keeps the groups it was
    # started with; restart it to pick up the new one.
    server.shell(
        name="restart user manager (groups changed)",
        commands=[f'systemctl restart "user@$(id -u {shlex.quote(user)}).service"'],
        _sudo=True, _if=kvm.did_change,
    )

    files.line(
        name="fstab: tmpfs /tmp", path="/etc/fstab",
        line=r"^tmpfs\s+/tmp\s+tmpfs\s",
        replace=f"tmpfs /tmp tmpfs {TMPFS_TMP_OPTS} 0 0", _sudo=True,
    )
    # findmnt reports size in KiB; remount whenever the live options drift.
    size_kib = TMPFS_SIZE_GIB * 1024**2
    server.shell(
        name="mount/remount tmpfs /tmp",
        commands=[
            "findmnt -no FSTYPE /tmp | grep -qx tmpfs || mount /tmp",
            f'opts=$(findmnt -no OPTIONS /tmp); '
            f'echo "$opts" | grep -q "\\bsize={size_kib}k\\b" && '
            f'echo "$opts" | grep -q nr_inodes || '
            f'mount -o remount,{TMPFS_TMP_OPTS} /tmp/',
        ],
        _sudo=True,
    )

    # Each rootless podman container uses keyring entries and an inotify
    # instance; allow ~2x the core count. Never lower existing values.
    ncpus = host.get_fact(Cpus) or 1
    reqs = {
        "kernel.keys.maxkeys": ncpus * 2,
        "kernel.keys.maxbytes": 1024 * ncpus * 2,
        "fs.inotify.max_user_instances": ncpus * 2,
    }
    actual = host.get_fact(Sysctl, keys=list(reqs))
    for key, required in reqs.items():
        current = actual.get(key)
        if current is not None and int(current) >= required:
            continue
        server.sysctl(
            name=f"sysctl {key} >= {required}", key=key, value=required,
            persist=True, persist_file="/etc/sysctl.d/99-hq-fuzzing.conf",
            _sudo=True,
        )

    # Reboot a wedged worker instead of leaving it hung. kernel.hardlockup_panic
    # only exists with a hardlockup detector (often absent in VMs).
    present = host.get_fact(Sysctl, keys=list(PANIC_SYSCTLS))
    for key, value in PANIC_SYSCTLS.items():
        if key not in present:
            print(f"WARNING [{host.name}]: no sysctl {key}, skipping", file=sys.stderr)
            continue
        server.sysctl(
            name=f"sysctl {key} = {value}", key=key, value=value,
            persist=True, persist_file="/etc/sysctl.d/99-hq-panic.conf",
            _sudo=True,
        )


def setup_user_systemd() -> None:
    """Enable lingering so user units run without a login session."""
    from pyinfra import host
    from pyinfra.facts.server import User
    from pyinfra.operations import server

    user = shlex.quote(host.get_fact(User))
    server.shell(
        name="enable linger",
        commands=[f"loginctl show-user {user} -p Linger --value | grep -qx yes || "
                  f"loginctl enable-linger {user}"],
        _sudo=True,
    )


def install_service(home: str, name: str, exec_start: str) -> None:
    """Install and start a user unit that restarts `exec_start` forever.

    Restarting (rather than giving up) also covers the worker's server being
    unreachable or its NFS share not being mounted yet.
    """
    from pyinfra.operations import files, systemd

    unit = f"""\
[Unit]
Description={name}
StartLimitIntervalSec=0

[Service]
WorkingDirectory={HQ_DIR}
Environment={HQ_SERVER_DIR_VAR}={HQ_SERVER_DIR}
ExecStart={exec_start}
Restart=always
RestartSec=2
LimitNOFILE=65535
TasksMax=infinity

[Install]
WantedBy=default.target
"""
    put = files.put(
        name=f"{name}.service", src=io.StringIO(unit),
        dest=f"{home}/.config/systemd/user/{name}.service",
    )
    systemd.service(
        name=f"restart {name} (unit changed)", service=f"{name}.service",
        user_mode=True, daemon_reload=True, restarted=True, _if=put.did_change,
    )
    systemd.service(
        name=f"{name} running", service=f"{name}.service",
        user_mode=True, running=True, enabled=True,
    )


def deploy_worker(home: str, nfs_src: str) -> None:
    from pyinfra.operations import files, server

    files.directory(name=f"{HQ_DIR} mountpoint", path=HQ_DIR, _sudo=True)
    files.line(
        name="fstab: NFS share", path="/etc/fstab",
        line=f"{nfs_src} {HQ_DIR} nfs defaults,_netdev 0 0",
        escape_regex_characters=True, _sudo=True,
    )
    setup_worker_host()
    server.shell(
        name=f"mount {HQ_DIR} (NFS)",
        commands=[f"mountpoint -q {HQ_DIR} || mount {HQ_DIR}"],
        _sudo=True,
    )
    setup_hq_env(home)
    install_service(home, "hq-worker", f"{HQ} worker start")


def deploy_local_worker(home: str) -> None:
    """Run a worker on the primary too, straight from the server's local HQ_DIR."""
    setup_worker_host()
    install_service(home, "hq-worker", f"{HQ} worker start")


def setup_primary_ssh(home: str, key: str, workers: list, cidr: str) -> None:
    """Let the primary `ssh WORKER`: the cluster key plus, per worker, an
    ~/.ssh/hq.d stanza and known_hosts pinned to the worker's host keys."""
    from pyinfra.facts.server import Command
    from pyinfra.operations import files

    ssh_dir = f"{home}/.ssh"
    hq_dir = f"{ssh_dir}/{SSH_HQ_DIR}"
    files.directory(name="~/.ssh dir", path=ssh_dir, mode="700")
    files.directory(name=f"~/.ssh/{SSH_HQ_DIR} dir", path=hq_dir, mode="700")
    files.put(name=f"~/.ssh/{SSH_KEY_NAME}", src=key,
              dest=f"{ssh_dir}/{SSH_KEY_NAME}", mode="600")
    files.put(name=f"~/.ssh/{SSH_KEY_NAME}.pub", src=f"{key}.pub",
              dest=f"{ssh_dir}/{SSH_KEY_NAME}.pub", mode="644")
    # Prepended: an Include after a Host line would only apply within it.
    files.block(
        name=f"~/.ssh/config: Include {SSH_HQ_DIR}", path=f"{ssh_dir}/config",
        content=f"Include {SSH_HQ_DIR}/*.conf", before=True, after=True,
        marker="# {mark} hq-deploy",
    )

    net = ipaddress.ip_network(cidr)
    for worker in workers:
        opts = ssh_config(worker.name)
        port = opts.get("port", "22")
        # The alias may resolve to an address the primary can't reach (e.g.
        # a VPN one); use the worker's own address in the cluster subnet.
        addrs = [
            str(iface.ip)
            for a in re.findall(r"\binet (\S+)", worker.get_fact(
                Command, command="ip -o -4 addr show scope global"))
            if (iface := ipaddress.ip_interface(a)).ip in net
        ]
        if not addrs:
            raise SystemExit(f"{worker.name!r} has no address in {cidr}")
        addr = opts.get("hostname") if opts.get("hostname") in addrs else addrs[0]

        known_host = addr if port == "22" else f"[{addr}]:{port}"
        host_keys = worker.get_fact(Command, command="cat /etc/ssh/ssh_host_*_key.pub")
        known_hosts = "".join(
            f"{known_host} {' '.join(k.split()[:2])}\n" for k in host_keys.splitlines() if k.strip()
        )
        files.put(
            name=f"known_hosts: {worker.name}", src=io.StringIO(known_hosts),
            dest=f"{hq_dir}/{worker.name}.known_hosts", mode="600",
        )
        files.put(
            name=f"ssh config: {worker.name}",
            src=io.StringIO(f"""\
Host {worker.name}
    HostName {addr}
    User {opts["user"]}
    Port {port}
    IdentityFile ~/.ssh/{SSH_KEY_NAME}
    IdentitiesOnly yes
    UserKnownHostsFile ~/.ssh/{SSH_HQ_DIR}/{worker.name}.known_hosts
    StrictHostKeyChecking yes
"""),
            dest=f"{hq_dir}/{worker.name}.conf", mode="600",
        )


def authorize_primary(home: str, pubkey: str, server_ip: str) -> None:
    """Accept the cluster key, from the primary only. Matching on the key's
    comment lets a rotated key replace the old line."""
    from pyinfra.operations import files

    comment = pubkey.split()[-1]
    files.directory(name="~/.ssh dir", path=f"{home}/.ssh", mode="700")
    files.line(
        name=f"authorized_keys: {comment}", path=f"{home}/.ssh/authorized_keys",
        # Not re.escape()d: grep warns about its `\-`; a stray `.` is harmless.
        line=f" {comment}$", replace=f'from="{server_ip}" {pubkey}',
    )


# --- launcher (direct invocation) -------------------------------------------

def bootstrap(argv: list[str]) -> int:
    """Parse the friendly CLI and re-enter via the pyinfra CLI, in-process."""
    import argparse

    p = argparse.ArgumentParser(prog="hq-deploy.py", description="Provision an hq cluster with pyinfra.")
    p.add_argument("hosts", nargs="+", help="ssh-config host aliases to provision")
    p.add_argument("--primary", required=True, help="host that runs the hq server")
    p.add_argument("--server-ip", help="override the discovered server IP")
    p.add_argument("--hq-url", default="https://github.com/It4innovations/hyperqueue/releases/download/v0.26.2/hq-v0.26.2-linux-x64.tar.gz",
                   help="hq binary or .tar.gz to install on the server if missing")
    p.add_argument("--worker-on-primary", action="store_true", help="also run a worker on the primary")
    p.add_argument("--comfy", action="store_true", help="install btop/htop, configure tmux, set timezone")
    p.add_argument("--timezone", help="timezone for --comfy (default: this machine's)")
    p.add_argument("--dry", action="store_true", help="preview changes without applying them")
    p.add_argument("pyinfra_args", nargs="*", help="extra pyinfra args (after --)")
    args = p.parse_args(argv)

    # The primary is always deployed: the workers' setup is useless without it.
    hosts = list(dict.fromkeys([args.primary, *args.hosts]))
    pa = [",".join(hosts), __file__, "--data", f"primary={args.primary}"]
    if args.server_ip:
        pa += ["--data", f"server_ip={args.server_ip}"]
    if args.hq_url:
        pa += ["--data", f"hq_url={args.hq_url}"]
    if args.worker_on_primary:
        pa += ["--data", "worker_on_primary=1"]
    if args.comfy:
        tz = args.timezone or local_timezone()
        if not tz:
            p.error("could not determine local timezone; pass --timezone")
        pa += ["--data", "comfy=1", "--data", f"timezone={tz}"]
    # -y also skips pyinfra's change detection, which is all --dry would show.
    pa.append("--dry" if args.dry else "-y")
    pa += args.pyinfra_args

    # Re-enter like the `pyinfra` console script (sets is_cli, sys.exit()s).
    from pyinfra_cli.main import main as pyinfra_main
    sys.argv = ["pyinfra", *pa]
    pyinfra_main()
    return 0


# pyinfra execs this file with is_cli=True; direct invocation bootstraps it.
import pyinfra  # noqa: E402

if pyinfra.is_cli:
    build_operations()
elif __name__ == "__main__":
    sys.exit(bootstrap(sys.argv[1:]))
