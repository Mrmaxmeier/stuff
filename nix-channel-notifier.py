#!/usr/bin/env python3
import sys
import time
import functools

import requests
import arrow
import sh

CHANNEL = sys.argv[1] if len(sys.argv) > 1 else "nixos-unstable"
GIT_REV_URL = f"https://channels.nixos.org/{CHANNEL}/git-revision"
REPO = "NixOS/nixpkgs"


s = requests.Session()

@functools.lru_cache(3)
def commit_timestamp(rev):
    r = s.get(f"https://api.github.com/repos/{REPO}/git/commits/{rev}")
    return arrow.get(r.json()["committer"]["date"])

def notify(message):
    title = "Nix Channel Update"
    try:
        sh.notify_send(title, message, urgency="low")
    except sh.ErrorReturnCode_1 as e:
        # NOTE: below is the implementation that I'd expect to work.
        # turns out we can ignore this entirely. Might break in the future though.
        if b"GDBus.Error:org.freedesktop.Notifications.Error.ExcessNotificationGeneration" in e.stderr:
            print(e.stderr.decode().strip())

print("Started ({})".format(time.strftime("%Y-%m-%d %H:%M:%S")))
current_rev = None
while True:
    try:
        r = s.get(GIT_REV_URL)
        rev = r.text.strip()
        if current_rev is None:
            print(f"initial rev: {rev}")
            ts = commit_timestamp(rev)
            print(f"timestamp: {ts} ({ts.humanize(arrow.now())})")
        elif rev != current_rev:
            print(f"New rev: {rev}")
            old_ts = commit_timestamp(current_rev)
            new_ts = commit_timestamp(rev)
            shortcode = rev[:7]
            text = f"{CHANNEL} advanced by {old_ts.humanize(new_ts, only_distance=True, granularity='hour')}\n"
            text += f"now at: {shortcode} ({new_ts.humanize(arrow.now(), granularity='hour')})"
            print(text)
            notify(text)
        current_rev = rev
    except requests.exceptions.ConnectionError as e:
        print(e)
        time.sleep(60)
    time.sleep(120)
