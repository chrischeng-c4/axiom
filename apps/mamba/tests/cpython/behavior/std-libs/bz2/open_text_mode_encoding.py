# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_text_mode_encoding"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open in wt/rt mode encodes and decodes unicode text with an explicit encoding"""
import bz2
import io

buf = io.BytesIO()
with bz2.open(buf, "wt", encoding="utf-8") as f:
    f.write("héllo wörld")
buf.seek(0)
with bz2.open(buf, "rt", encoding="utf-8") as f:
    text = f.read()
assert text == "héllo wörld", f"text mode = {text!r}"
print("open_text_mode_encoding OK")
