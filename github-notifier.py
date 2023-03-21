#!/usr/bin/python3
from pprint import pprint
import sys
import os
import time
import json

import requests
import arrow
import sh

if len(sys.argv) > 1:
    cfg_path = sys.argv[1]
else:
    cfg_path = os.path.expanduser("~") + "/.config/github-notifier.json"

CLIENT_ID = "2cdcb6911fa2bd088826"

if not os.path.isfile(cfg_path):
    print(cfg_path, "missing, generating token via oauth device flow...")

    res = requests.post("https://github.com/login/device/code", headers={
        'Accept': 'application/json'
    }, data={
        'client_id': CLIENT_ID,
        'scope': 'notifications read:user',
    })
    data = res.json()
    # print(json.dumps(data, indent=4))
    print(f"User Verification Code: {data['user_code']}")

    print("Go to this url https://github.com/login/device and enter the code.")
    sh.Command("xdg-open")("https://github.com/login/device")
    print("Press any key to continue")
    input()


    res = requests.post('https://github.com/login/oauth/access_token', headers={
        'Accept': 'application/json'
    }, data={
        'client_id': CLIENT_ID,
        'device_code': data['device_code'],
        'grant_type': 'urn:ietf:params:oauth:grant-type:device_code'
    })
    # print(json.dumps(res.json(), indent=4))

    token = res.json()['access_token']

    # print("\n\nUser Info:\n")
    res = requests.get(
        "https://api.github.com/user", headers={'Authorization': 'token ' + res.json()['access_token']})

    # print(json.dumps(res.json(), indent=4) + '\n\n\n')
    username = res.json()["login"]

    print(f"Check your authorization at https://github.com/settings/connections/applications/{CLIENT_ID}")

    with open(cfg_path, "w") as f:
        json.dump({
            "token": token,
            "username": username,
            "data": {},
            "last_event": str(arrow.now())
        }, f)

with open(cfg_path, "r") as f:
    cfg = json.load(f)

etag = None
last_event = arrow.get(cfg["last_event"])
auth = "token " + cfg["token"]

def actorName(d):
    return d["actor"]["login"]

def repoName(d):
    return d["repo"]["name"]

def createDeleteEvent(d):
    s = "{} {} {} ".format(actorName(d), "created" if d["type"] == "CreateEvent" else "deleted", d["payload"]["ref_type"])
    if d["payload"]["ref_type"] == "repository":
        s += d["repo"]["name"]
    elif d["payload"]["ref_type"] in ["branch", "tag"]:
        s += d["payload"]["ref"]
    else:
        s += d["payload"]["ref_type"]
    return s

def shortened(msg, length=55):
    msg = msg.strip("\n")
    if len(msg.split("\n")) < 2 and len(msg) <= length:
        return msg
    return msg.split("\n")[0][:length] + " [...]"

known_types = {
    "PushEvent": lambda d: "\n".join([c["author"]["name"] + ": " + shortened(c["message"]) for c in d["payload"]["commits"]]) + "\n@" + d["repo"]["name"],
    "WatchEvent": lambda d: "{} starred {}".format(actorName(d), d["repo"]["name"]),
    "CreateEvent": createDeleteEvent,
    "DeleteEvent": createDeleteEvent,
    "IssuesEvent": lambda d: "{} {} '{}'\n@{}".format(actorName(d), d["payload"]["action"], shortened(d["payload"]["issue"]["title"]), repoName(d)),
    "IssueCommentEvent": lambda d: "{} commented '{}'\non '{}'\n@{}".format(actorName(d), shortened(d["payload"]["comment"]["body"]), shortened(d["payload"]["issue"]["title"]), repoName(d)),
    "CommitCommentEvent": lambda d: "{} commented '{}'\non '{}'\n@{}".format(actorName(d), shortened(d["payload"]["comment"]["body"]), shortened(d["payload"]["comment"]["path"]), repoName(d)),
    "PullRequestEvent": lambda d: "{} {} '{}'\n@{}".format(actorName(d), d["payload"]["action"], shortened(d["payload"]["pull_request"]["title"]), repoName(d)),
    "ForkEvent": lambda d: "{} forked {}".format(actorName(d), repoName(d)),
    "ReleaseEvent": lambda d: "{} {} '{}'\n@{}".format(actorName(d), d["payload"]["action"], shortened(d["payload"]["release"]["name"]), repoName(d)),
    "GollumEvent": lambda d: "{}\n{}\n@{}".format(actorName(d), "\n".join(["{} '{}'".format(p['action'], p['title']) for p in d['payload']['pages']]), repoName(d)),
    "PullRequestReviewCommentEvent": lambda d: "{} reviewed '{}'".format(actorName(d), shortened(d["payload"]["pull_request"]["title"])),
    "MemberEvent": lambda d: "{} {} {}\n@{}".format(actorName(d), d["payload"]["action"], d["payload"]["member"]["login"], repoName(d)),
    "PublicEvent": lambda d: "{} made {} public".format(actorName(d), repoName(d)),
}

def tostring(d):
    if d["type"] in known_types:
        return known_types[d["type"]](d), True
    return "'{}'".format(d["type"]), False

def notify(d):
    title = "Github Notification"
    message, type_found = tostring(d)
    private = not d["public"]
    if not type_found:
        pprint(d)
    print(title + " - Private" if private else title)
    print(message + "\n")
    sh.notify_send(title, message, urgency="normal" if private else "low")

print("Started ({})".format(time.strftime("%Y-%m-%d %H:%M:%S")))
sh.notify_send('github_notifier started', urgency="low")
while True:
    headers = {"Accept": "application/vnd.github.v3+json", "Authorization": auth}
    if etag:
        headers["If-None-Match"] = etag
    try:
        r = requests.get("https://api.github.com/users/" + cfg["username"] + "/received_events", headers=headers)
    except requests.exceptions.ConnectionError as e:
        print(e)
        time.sleep(60)
        continue
    if r.status_code == 304:
        time.sleep(30)
        continue

    for elem in r.json()[::-1]:
        if (last_event - arrow.get(elem["created_at"])).total_seconds() < 0:
            last_event = arrow.get(elem["created_at"])
            notify(elem)
            cfg["last_event"] = str(last_event)
            with open(cfg_path, "w") as f:
                json.dump(cfg, f)

    etag = r.headers["etag"]
    try:
        time.sleep(int(r.headers["x-poll-interval"]))
    except KeyboardInterrupt:
        exit()
