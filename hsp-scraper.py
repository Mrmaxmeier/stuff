#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["requests", "arrow", "jsondiff"]
# ///

import json
import os
from pathlib import Path

import requests
import arrow
import jsondiff

URL = "https://hsp-sv.uni-saarland.de/LANG-DEU/Home/KursListe"
NEEDLE = "    var data = {"
CHANGELOG_PATH = Path(os.environ.get("CHANGELOG_PATH", "hsp-changes.jsonl"))
SNAPSHOT_PATH = Path(os.environ.get("SNAPSHOT_PATH", "hsp-snapshot.json"))

def get_data():
    """
    {
    "list": [
        [...]
        {
        "kursLogo": "/temp/res/356d8e31-f214-4abb-93ea-36ece535d6a3.jpg_200.jpg?ts=638476664739915340",
        "TerminID": null,
        "CourseID": "55002",
        "WochentagInt": 1,
        "Sport": "Rettungsschwimmabzeichen Silber",
        "Wochentag": "Montag",
        "Kursname": "Rettungsschwimmabzeichen Silber<br> Kooperation",
        "WannUndZeitraum": "Mo, 20:00 - 21:00 Uhr",
        "Ort": "KOI Wasserwelt (Homburg)",
        "Status": "<div class='free'>Buchbar</div>",
        "KursLogo": "/temp/res/356d8e31-f214-4abb-93ea-36ece535d6a3.jpg_200.jpg?ts=638476664739915340",
        "Start": "2025-11-10T20:00:01.335",
        "isOver": false,
        "IsKursleiter": false,
        "TeilnehmerAnzahl": "5 frei",
        "AnbieterName": "Hochschulsport"
        }
    ],
    "sortIndex": 7,
    "sortIndexFixed": 3,
    "group1Index": 3,
    "hideIndex": -1
    }
    """
    res = requests.get(URL)
    data = None
    for line in res.text.splitlines():
        if line.startswith(NEEDLE):
            data = json.loads(line.removeprefix(NEEDLE[:-1]).removesuffix(";"))
    assert data is not None
    courses = data["list"]
    courses = sorted(courses, key=lambda c: int(c["CourseID"]))
    return courses

def load_snapshot():
    if not SNAPSHOT_PATH.exists():
        return []
    with open(SNAPSHOT_PATH, "r") as f:
        return json.load(f)

def save_snapshot(courses):
    with open(SNAPSHOT_PATH, "w") as f:
        json.dump(courses, f)

if __name__ == "__main__":
    snapshot = load_snapshot()
    courses = get_data()
    changes = jsondiff.diff(snapshot, courses, marshal=True)
    if changes:
        print(f"{len(changes) = }")
        changes = {"timestamp": arrow.now().isoformat(), "changes": changes}
        with open(CHANGELOG_PATH, "a") as f:
            f.write(json.dumps(changes) + "\n")
    save_snapshot(courses)
