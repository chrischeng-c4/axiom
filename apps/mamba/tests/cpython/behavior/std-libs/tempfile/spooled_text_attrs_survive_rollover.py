# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_text_attrs_survive_rollover"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: a text spool (mode='w+', encoding='utf-8') round-trips strings and keeps encoding='utf-8'/errors='strict' across rollover"""
import tempfile

t = tempfile.SpooledTemporaryFile(mode="w+", max_size=10, encoding="utf-8")
t.write("abc\n")
t.seek(0)
assert t.read() == "abc\n", "text round-trip before rollover"
assert not t._rolled, "small text stays in memory"
assert t.mode == "w+" and t.name is None
assert t.encoding == "utf-8", f"encoding = {t.encoding!r}"
assert t.errors == "strict", f"errors = {t.errors!r}"
t.write("xyzzy\n" * 4)  # push over max_size
t.seek(0)
assert t.read() == "abc\n" + "xyzzy\n" * 4, "content survives rollover"
assert t._rolled and t.mode == "w+" and t.name is not None
assert t.encoding == "utf-8" and t.errors == "strict"
t.close()
print("spooled_text_attrs_survive_rollover OK")
