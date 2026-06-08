# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "real_world"
# case = "csv_style_buffer_roundtrip"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: a realistic in-memory serializer flow: build rows into a StringIO, rewind, and parse them back into the original records"""
import io

# An in-memory serializer flow: serialize records into a StringIO with a
# simple pipe-delimited layout, rewind, then parse the lines back into the
# original records — the shape of a logging/CSV-writer round-trip.
records = [
    ("alice", "30", "engineer"),
    ("bob", "25", "designer"),
    ("carol", "41", "manager"),
]

buf = io.StringIO()
for name, age, role in records:
    buf.write("|".join((name, age, role)))
    buf.write("\n")

buf.seek(0)
parsed = [tuple(line.rstrip("\n").split("|")) for line in buf]
assert parsed == records, f"round-trip mismatch: {parsed!r}"

# getvalue() still exposes the full serialized payload after iteration.
assert buf.getvalue().count("\n") == len(records), "row count mismatch"

print("csv_style_buffer_roundtrip OK")
