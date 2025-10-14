#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["requests", "sh"]
# ///

import time

import requests
import sh

EPS = 25
INFO_PPM_THRESHOLD = 850
WARN_PPM_THRESHOLD = 1000

def notify(measurements, urgency="low"):
    title = "Co2 Meter"
    avg = sum(measurements) / len(measurements)
    err = round(max(
        avg-min(measurements),
        max(measurements)-avg
    ))
    avg = round(avg)
    message = f"{avg} ppm ±{err}" if err else f"{avg} ppm"
    sh.notify_send(title, message, urgency=urgency)

print("Started ({})".format(time.strftime("%Y-%m-%d %H:%M:%S")))

def fetch():
    try:
        res = requests.get("http://100.102.58.110:9123/metrics")
        assert res.status_code == 200
    except Exception as e:
        print(e)
        return None
    measurement = next(line for line in res.text.split("\n") if line.startswith("meter_co2_ppm"))
    measurement = int(measurement.split()[1])
    return measurement

state = None
measurement = fetch()
roller = [measurement]
notify(roller, urgency="low")

while True:
    measurement = fetch()
    if measurement is not None:
        roller.append(measurement)
        roller = roller[-4:]
        avg = sum(roller) / len(roller)
        print(f"{measurement = }")
        print(f"{avg         = }")
        if avg > INFO_PPM_THRESHOLD+EPS and state == None:
            state = "INFO"
            notify(roller, urgency="low")
        elif avg > WARN_PPM_THRESHOLD+EPS and state != "WARN":
            state = "WARN"
            notify(roller, urgency="normal")
        elif avg < WARN_PPM_THRESHOLD-EPS and state == "WARN":
            state = "INFO"
            notify(roller, urgency="low")
        elif avg < INFO_PPM_THRESHOLD-EPS and state != None:
            state = None
            notify(roller, urgency="low")
    try:
        time.sleep(25)
    except KeyboardInterrupt:
        exit()
