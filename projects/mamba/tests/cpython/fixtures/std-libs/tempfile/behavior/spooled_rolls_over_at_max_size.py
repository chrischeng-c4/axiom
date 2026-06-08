# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_rolls_over_at_max_size"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: a SpooledTemporaryFile stays in memory (_rolled False) until a write exceeds max_size, then rolls to disk and its content survives"""
import tempfile

_spooled = tempfile.SpooledTemporaryFile(max_size=10)
_spooled.write(b"short")
assert not _spooled._rolled, "not yet spilled to disk"
_spooled.write(b"x" * 20)  # exceed max_size=10
assert _spooled._rolled, "rolled after exceeding max_size"
_spooled.seek(0)
_read = _spooled.read()
assert _read.startswith(b"short"), f"spooled content = {_read[:5]!r}"
_spooled.close()
print("spooled_rolls_over_at_max_size OK")
