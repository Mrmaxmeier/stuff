#!/usr/bin/env python3
# Strip hidden anchor tags from HedgeDoc rich text copies, which could
# be used to link i.e. an email body back to the HedgeDoc instance

import subprocess
from bs4 import BeautifulSoup

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

clip = Gtk.Clipboard.get(Gdk.atom_intern("CLIPBOARD", True))


sel = clip.wait_for_contents(Gdk.atom_intern("text/html", True))
if sel is None:
    print("targets:")
    print(*clip.wait_for_targets()[1], sep = "\n")
    raise RuntimeError("no rich text in clipboard")

html = sel.get_data()
soup = BeautifulSoup(html, 'html.parser')

domains = set()
found_anchor_elems = False

for anchor in soup.find_all("a", class_="anchor"):
    href = anchor.attrs["href"]
    domain = href[:href.index("#")]
    found_anchor_elems = True
    if domain:
        domains.add(domain)
        anchor.attrs["href"] = href[href.index("#"):]

if domains:
    print(f"stripped anchor tags for {domains = }")
    # gtk 3 bindings don't expose APIs to set contents for specific clipboard selections
    # https://stackoverflow.com/questions/25151437/copy-html-to-clipboard-with-pygobject/25152058#25152058
    cmd = ["xclip", "-sel", "clip", "-t", "text/html", "-f"]
    subprocess.check_output(cmd, input=str(soup), text=True)
elif found_anchor_elems:
    print("already stripped")
else:
    print("no anchor elements found")