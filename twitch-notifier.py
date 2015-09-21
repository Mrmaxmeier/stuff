#!/usr/bin/python3

import os
import sys
import json
import requests
import time
import sh
from pprint import pprint

if len(sys.argv) > 1:
	cfg_path = sys.argv[1]
else:
	cfg_path = os.path.expanduser("~") + "/.config/twitch-notifier.json"


if not os.path.isfile(cfg_path):
	print(cfg_path, "missing")
	username = input("Username> ")
	auth = input("Auth-Token> ")
	with open(cfg_path, "w") as f:
		json.dump({
			"auth": auth,
			"username": username,
			"data": {}
		}, f)



delay = 90
blocking = ["i3lock"]
required = ["i3"]

with open(cfg_path, "r") as f:
	data = json.load(f)

headers = {
	"Accept": "application/vnd.twitchtv.v3+json",
	"Authorization": "OAuth " + data["auth"]
}

tobenotified = []
notified_start = False


print("Started ({})".format(time.strftime("%Y-%m-%d %H:%M:%S")))

while True:
	try:
		online = requests.get("https://api.twitch.tv/kraken/streams/followed?limit=100", headers=headers).json()["streams"]
	except requests.exception.ConnectionError as e:
		print(e)
		time.sleep(delay)
		continue
	for channel in online:
		game = channel["game"]
		username = channel["channel"]["name"]
		display = channel["channel"]["display_name"]
		# print(game, username, display)
		if username in data["data"]:
			if data["data"][username]["notify"] and data["data"][username]["last_processed"] < time.time() - delay * 5:
				tobenotified.append(display)
			data["data"][username]["last_processed"] = time.time()
		else:
			data["data"][username] = {
				"notify": True,
				"last_processed": time.time()
			}
			tobenotified.append(display)
			print("[+] {} ({}) {}".format(display, game, time.strftime("%Y-%m-%d %H:%M:%S")))

	online_display = [i["channel"]["display_name"] for i in online]
	tobenotified = [i for i in tobenotified if i in online_display]

	blocked = False
	for pname in blocking:
		try:
			sh.pgrep(pname)
			blocked = True
			print("blocked by", pname)
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
		if len(tobenotified) > 2:
			l = [u[0].upper() + u[1:] for u in tobenotified]
		else:
			def f(u):
				for channel in online:
					if channel["channel"]["display_name"] == u:
						return "{}{} ({})\n".format(u[0].upper(), u[1:], channel["game"])
			l = map(f, tobenotified)
		sh.notify_send("Twitch", " ".join(l))
		tobenotified = []
	if not blocked and not notified_start:
		sh.notify_send('twitch_notifier started')
		notified_start = True

	with open(cfg_path, "w") as f:
		json.dump(data, f)

	time.sleep(delay)
