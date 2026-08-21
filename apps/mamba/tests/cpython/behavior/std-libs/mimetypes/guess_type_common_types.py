# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_common_types"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: common filenames map to their IANA type with no encoding: .html/.css/.js/.png/.jpg/.json/.pdf"""
import mimetypes

cases = [
    ("index.html", "text/html", None),
    ("style.css", "text/css", None),
    ("script.js", "text/javascript", None),
    ("image.png", "image/png", None),
    ("image.jpg", "image/jpeg", None),
    ("data.json", "application/json", None),
    ("doc.pdf", "application/pdf", None),
]
for fname, etype, eenc in cases:
    t, e = mimetypes.guess_type(fname)
    assert t == etype, f"{fname!r}: type = {t!r}, expected {etype!r}"
    assert e == eenc, f"{fname!r}: encoding = {e!r}"
print("guess_type_common_types OK")
