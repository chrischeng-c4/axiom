# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "string_form_8_4_4_4_12"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: str(UUID) is the canonical 8-4-4-4-12 dash-grouped 36-char form"""
import uuid

s = str(uuid.uuid4())
assert len(s) == 36, f"str len = {len(s)!r}"
parts = s.split("-")
assert len(parts) == 5, f"UUID parts = {len(parts)!r}"
assert [len(p) for p in parts] == [8, 4, 4, 4, 12], f"part lens = {[len(p) for p in parts]!r}"
print("string_form_8_4_4_4_12 OK")
