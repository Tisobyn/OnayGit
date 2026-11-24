#!/bin/sh
# Dev helper: run the app with a demo HOME and save a window screenshot to $1.
S=/private/tmp/claude-501/-Users-tisobyn--claude-spares-spare-1h3I0H/0f848c29-eb82-41ff-bdab-1d0d391e22b1/scratchpad
pkill onaygit; (ONAYGIT_SCREEN=$2 HOME=${H:-$S/home} /Users/tisobyn/Developer/OnayGit/target/release/onaygit >/dev/null 2>&1 &); sleep 6
ID=$(swift -e 'import CoreGraphics
for w in CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as! [[String:Any]] { if (w["kCGWindowOwnerName"] as? String ?? "").lowercased().contains("onaygit"), let b = w["kCGWindowBounds"] as? [String:Any], (b["Height"] as! Int) > 100 { print(w["kCGWindowNumber"]!) } }')
screencapture -l $ID -o $S/$1.png
