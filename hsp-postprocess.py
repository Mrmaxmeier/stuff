#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["jsondiff"]
# ///

from collections import defaultdict
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Tuple
from pprint import pprint
import json
import jsondiff

@dataclass
class Course:
    id: str
    name: str
    place: str
    date: str
    free_spots: int
    registration_open: bool

@dataclass
class Snapshot:
    timestamp: str
    courses: List[Course]

def parse_snapshot(timestamp: str, snapshot: dict) -> Snapshot:
    courses = []
    skip_courses = ["key competencies", "Uni-Fit – Einführungskurs", "Uni-Fit – Introductory Course", "Kunstrasenplatz"]
    for el in snapshot:
        # pprint(el)
        if el["Start"].startswith("9999-"): continue
        if "Saarbrücken" not in el["Ort"]: continue
        free_spots = 0
        if " frei" in el.get("TeilnehmerAnzahl", ""):
            free_spots = int(el["TeilnehmerAnzahl"].removesuffix(" frei"))
        name = el["Kursname"]
        name = name.replace('<span style="color:#FFB000"><b> – NEU –</B>', "")
        name = name.replace('<span style="color:#c82254"><b>– fit4more –</B>', "")
        name = name.replace('<span style="color:#FFB000">', "")
        name = name.replace("<br>", ":", 1).replace("<br>", "").replace("</br>", "")
        name = name.strip()
        name = name.rstrip(":")
        if any(x.lower() in name.lower() for x in skip_courses):
            continue
        courses.append(Course(
            id=el["CourseID"],
            name=name,
            place=el["Ort"],
            date=el["WannUndZeitraum"],
            free_spots=free_spots,
            registration_open=el["Status"] == "<div class='free'>Buchbar</div>"
        ))
    return Snapshot(timestamp=timestamp, courses=courses)
    

def get_snapshots(jsonl_file):
    cur = []
    res = []
    for line in jsonl_file.readlines():
        data = json.loads(line)
        timestamp, changes = data["timestamp"], data["changes"]
        cur = jsondiff.patch(cur, changes, marshal=True)
        res.append(parse_snapshot(timestamp, cur))
    return res


@dataclass
class Change:
    timestamp: str
    course_id: str
    course_name: str
    free_spots_old: int | None
    free_spots_new: int | None

@dataclass
class CourseReport:
    course: Course
    spots_timeline: List[Tuple[str, int]]

@dataclass
class Report:
    changelog: List[Change]
    courses: List[CourseReport]


def generate_report(snapshots: List[Snapshot]) -> Report:
    changelog = []
    course_spots = defaultdict(lambda: -1)
    course_ts = defaultdict(list)
    for snapshot in snapshots:
        for course in snapshot.courses:
            if course_spots[course.id] != course.free_spots:
                if course_spots[course.id] != -1:
                    print(f"{snapshot.timestamp} {course_spots[course.id]:>3d} ~> {course.free_spots:>3d} {course.name}")
                course_ts[course.id].append((snapshot.timestamp, course.free_spots))
            course_spots[course.id] = course.free_spots

    for a, b in zip(snapshots, snapshots[1:]):
        cas = {c.id: c for c in a.courses}
        cbs = {c.id: c for c in b.courses}
        ids = sorted(set(cas.keys()) | set(cbs.keys()))
        for course_id in ids:
            ca = cas.get(course_id, None)
            cb = cbs.get(course_id, None)
            name = ca.name if ca is not None else cb.name
            free_spots_old = ca.free_spots if ca is not None else None
            free_spots_new = cb.free_spots if cb is not None else None
            if cb is None:
                print(f"{b.timestamp} Removed: {course_id} {name}")
            elif ca is None:
                print(f"{b.timestamp} Added: {course_id} {name}")
            changelog.append(Change(timestamp=b.timestamp, course_id=course_id, course_name=name, free_spots_old=free_spots_old, free_spots_new=free_spots_new))

    course_reports = [
        CourseReport(course=course, spots_timeline=course_ts[course.id])
        for course in snapshots[-1].courses
    ]
    return Report(changelog=changelog, courses=course_reports)


if __name__ == "__main__":
    import sys
    assert sys.argv[1:], "USAGE: hsp-postprocess.py <changes.jsonl>"
    path = Path(sys.argv[1])
    if not path.is_file():
        print(f"file not found: {path}")
        sys.exit(1)

    with open(path, "r") as f:
        snapshots = get_snapshots(f)

    report = generate_report(snapshots)
    # pprint(report)
    if sys.argv[2:]:
        with open(sys.argv[2], "w") as f:
            json.dump(asdict(report), f)
