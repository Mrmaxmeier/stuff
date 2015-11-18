#!/usr/bin/python3

import requests
import time
import arrow
import json
import sys
import os
import getpass
import sh
from pprint import pprint

if len(sys.argv) > 1:
	cfg_path = sys.argv[1]
else:
	cfg_path = os.path.expanduser("~") + "/.config/github-notifier.json"

if not os.path.isfile(cfg_path):
	print(cfg_path, "missing")
	username = input("Username: ")
	pw = getpass.getpass()
	with open(cfg_path, "w") as f:
		json.dump({
			"login": (username, pw),
			"username": username,
			"data": {},
			"last_event": str(arrow.now())
		}, f)

with open(cfg_path, "r") as f:
	cfg = json.load(f)

if any([len(s) < 1 for s in cfg["login"]]):
	login = None
else:
	login = cfg["login"]

etag = None
last_event = arrow.get(cfg["last_event"])
auth = requests.auth.HTTPBasicAuth(login[0], login[1]) if login else None

blocking = ["i3lock"]
required = ["i3"]

def actorName(d):
	return d["actor"]["login"]

def repoName(d):
	return d["repo"]["name"]

def createDeleteEvent(d):
	s = "{} {} ".format(actorName(d), "created" if d["type"] == "CreateEvent" else "deleted")
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
	else:
		return msg.split("\n")[0][:length] + " [...]"

known_types = {
	"PushEvent": lambda d: "\n".join([c["author"]["name"] + ": " + shortened(c["message"]) for c in d["payload"]["commits"]]) + "\n@" + d["repo"]["name"],
	"WatchEvent": lambda d: "{} starred {}".format(actorName(d), d["repo"]["name"]),
	"CreateEvent": createDeleteEvent,
	"DeleteEvent": createDeleteEvent,
	"IssuesEvent": lambda d: "{} {} '{}'\n@{}".format(actorName(d), d["payload"]["action"], shortened(d["payload"]["issue"]["title"]), repoName(d)),
	"IssueCommentEvent": lambda d: "{} commented '{}'\non '{}'\n@{}".format(actorName(d), shortened(d["payload"]["comment"]["body"]), shortened(d["payload"]["issue"]["title"]), repoName(d)),
	"PullRequestEvent": lambda d: "{} {} '{}'\n@{}".format(actorName(d), d["payload"]["action"], shortened(d["payload"]["pull_request"]["title"]), repoName(d)),
	"ForkEvent": lambda d: "{} forked {}".format(actorName(d), repoName(d)),
	"ReleaseEvent": lambda d: "{} {} '{}'\n@{}".format(actorName(d), d["payload"]["action"], shortened(d["payload"]["release"]["name"]), repoName(d)),
	"GollumEvent": lambda d: "{}\n{}\n@{}".format(actorName(d), "\n".join(["{} '{}'".format(p['action'], p['title']) for p in d['payload']['pages']]), repoName(d))
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

tobenotified = []
notified_start = False

print("Started ({})".format(time.strftime("%Y-%m-%d %H:%M:%S")))
while True:
	headers = {"Accept": "application/vnd.github.v3+json"}
	if etag:
		headers["If-None-Match"] = etag
	try:
		r = requests.get("https://api.github.com/users/" + cfg["username"] + "/received_events", headers=headers, auth=auth)
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
			tobenotified.append(elem)
			cfg["last_event"] = str(last_event)
			with open(cfg_path, "w") as f:
				json.dump(cfg, f)


	blocked = False
	for pname in blocking:
		try:
			sh.pgrep(pname)
			blocked = True
			if len(tobenotified) > 0:
				print("{} notifications blocked by {}".format(len(tobenotified), pname))
			break
		except sh.ErrorReturnCode_1:
			pass
	for pname in required:
		try:
			sh.pgrep(pname)
		except sh.ErrorReturnCode_1:
			blocked = True
			print(pname, "required")
			break

	if not blocked and tobenotified:
		for elem in tobenotified:
			notify(elem)
		tobenotified = []
	if not blocked and not notified_start:
		sh.notify_send('github_notifier started', urgency="low")
		notified_start = True

	etag = r.headers["etag"]
	try:
		time.sleep(int(r.headers["x-poll-interval"]))
	except KeyboardInterrupt:
		exit()
