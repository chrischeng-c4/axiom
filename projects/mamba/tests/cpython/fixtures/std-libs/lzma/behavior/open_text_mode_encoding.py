# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "open_text_mode_encoding"
# subject = "lzma.open"
# kind = "semantic"
# xfail = "lzma.open is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:81-82)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.open: lzma.open in text mode ('wt'/'rt') honors the encoding and round-trips a unicode string"""
import lzma


import io
buf = io.BytesIO()
with lzma.open(buf, "wt", encoding="utf-8") as f:
    f.write("unicode: zh-text")
buf.seek(0)
with lzma.open(buf, "rt", encoding="utf-8") as f:
    text = f.read()
assert text == "unicode: zh-text", f"text mode = {text!r}"
print("open_text_mode_encoding OK")
