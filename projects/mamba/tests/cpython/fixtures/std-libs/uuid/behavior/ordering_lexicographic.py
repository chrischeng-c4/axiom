# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "ordering_lexicographic"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUIDs order by their 128-bit int: the all-zero-but-one value is less than the all-ones value"""
import uuid

small = uuid.UUID("00000000-0000-0000-0000-000000000001")
large = uuid.UUID("ffffffff-ffff-ffff-ffff-ffffffffffff")
assert small < large, "UUID ordering"
assert large > small, "UUID ordering (reverse)"
print("ordering_lexicographic OK")
