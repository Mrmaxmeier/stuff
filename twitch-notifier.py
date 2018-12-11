#!/usr/bin/python3

import os
import sys
import json
import time
from pprint import pprint

import requests
import sh

if len(sys.argv) > 1:
	cfg_path = sys.argv[1]
else:
	cfg_path = os.path.expanduser("~") + "/.config/twitch-notifier.json"


if not os.path.isfile(cfg_path):
	print(cfg_path, "missing")
	username = input("Username> ")
	print("Press enter to generate new auth token...")
	auth = input("Auth-Token> ")
	while not auth:
		print("requesting auth token")
		payload = {
			"response_type": "token",
			"client_id": input("Client ID> "),
			"redirect_uri": "http://localhost",
			"scope": "user_read",
		}
		params = "&".join([key + "=" + item for key, item in payload.items()])
		print("https://api.twitch.tv/kraken/oauth2/authorize?" + params)
		auth = input("Auth-Token> ")

	with open(cfg_path, "w") as f:
		json.dump({
			"auth": auth,
			"username": username,
			"data": {}
		}, f)



delay = 90
blocking = ["i3lock"]

with open(cfg_path, "r") as f:
	data = json.load(f)

headers = {
	"Accept": "application/vnd.twitchtv.v3+json",
	"Authorization": "OAuth " + data["auth"]
}

tobenotified = []


print("Started ({})".format(time.strftime("%Y-%m-%d %H:%M:%S")))
sh.notify_send('twitch_notifier started', urgency="low")

while True:
	try:
		res = requests.get("https://api.twitch.tv/kraken/streams/followed?limit=50", headers=headers).json()
		if 'streams' in res:
			online = res['streams']
		else:
			print(res)
			time.sleep(delay)
			continue
	except Exception as e:
		print(e)
		sh.notify_send("Twitch", str(e), urgency="low")
		time.sleep(delay)
		continue
	except requests.exceptions.ConnectionError as e:
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
				print("[*] {} ({}) {}".format(display, game, time.strftime("%Y-%m-%d %H:%M:%S")))
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
			if len(tobenotified) > 0:
				print("{} notifications blocked by {}".format(len(tobenotified), pname))
			break
		except sh.ErrorReturnCode_1:
			pass

	if not blocked and tobenotified:
		if len(tobenotified) > 2:
			l = [u[0].upper() + u[1:] for u in tobenotified]
		else:
			def f(u):
				for channel in online:
					if channel["channel"]["display_name"] == u:
						return "{}{} ({})\n".format(u[0].upper(), u[1:], channel["game"])
			l = map(f, tobenotified)
		sh.notify_send("Twitch", " ".join(l), urgency="low")
		tobenotified = []

	with open(cfg_path, "w") as f:
		json.dump(data, f)

	time.sleep(delay)
