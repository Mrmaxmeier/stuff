#!/usr/bin/python3

import os
import json
import requests
import time
import sh
from pprint import pprint

cfg_path = os.path.expanduser("~") + "/.config/twitch-nofifier.json"


if not os.path.isfile(cfg_path):
	username = input("Username> ")
	auth = input("Auth-Token> ")
	with open(cfg_path, "w") as f:
		json.dump({
			"auth": auth,
			"username": username,
			"data": {}
		}, f)



delay = 60
blocking = ["i3lock"]

with open(cfg_path, "r") as f:
	data = json.load(f)


headers = {
	"Accept": "application/vnd.twitchtv.v3+json",
	"Authorization": "OAuth " + data["auth"]
}

tobenotified = []
while True:
	online = requests.get("https://api.twitch.tv/kraken/streams/followed?limit=100", headers=headers).json()["streams"]
	for channel in online:
		game = channel["game"]
		username = channel["channel"]["name"]
		display = channel["channel"]["display_name"]
		# print(game, username, display)
		if username in data["data"]:
			if data["data"][username]["notify"] and data["data"][username]["last_processed"] < time.time() - delay * 2:
				tobenotified.append(display)
			data["data"][username]["last_processed"] = time.time()
		else:
			data["data"][username] = {
				"notify": True,
				"last_processed": time.time()
			}
			tobenotified.append(display)

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

	if not blocked and tobenotified:
		l = [u[0].upper() + u[1:] for u in tobenotified]
		sh.notify_send("Twitch", " ".join(l))
		tobenotified = []

	with open(cfg_path, "w") as f:
		json.dump(data, f)

	time.sleep(delay)
