#!/usr/bin/env python3
# Extract images from Excalidraw clipboard data

import json
import subprocess
import base64

from pathlib import Path

out = Path("/tmp/excalidraw-clipboard-images")
out.mkdir(exist_ok=True)


data = subprocess.check_output(["wl-paste"]).decode().strip()
try:
    data = json.loads(data)
    assert data["type"] == "excalidraw/clipboard"
except Exception as e:
    print("Unexpected clipboard contents:")
    print(data[:100])
    raise e

#for element in data["elements"]:
#    if element["type"] == "image":
#        print(element["fileId"])

for file in data["files"].values():
    if file["mimeType"] == "image/png":
        id = file["id"]
        dataURL = file["dataURL"]
        assert dataURL.startswith("data:image/png;base64,")
        dataURL = dataURL[len("data:image/png;base64,"):]
        png_data = base64.b64decode(dataURL)
        png_path = out / f"{id}.png"
        print(f"saving {png_path}")
        with open(png_path, "wb") as f:
            f.write(png_data)
    else:
        print(f"skipping file {file['mimeType']}")
