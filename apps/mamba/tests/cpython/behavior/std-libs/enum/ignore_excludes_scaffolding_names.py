# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "ignore_excludes_scaffolding_names"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: _ignore_ lists names that are class-body scaffolding only: they (and a loop variable) do not become members, while names assigned via vars() do"""
import enum

class Days(enum.Enum):
    _ignore_ = "Days i"
    Days = vars()
    for i in range(1, 4):
        Days["DAY_%d" % i] = i

names = sorted(m.name for m in Days)
assert names == ["DAY_1", "DAY_2", "DAY_3"]
assert "Days" not in names
assert "i" not in names
assert "_ignore_" not in names

print("ignore_excludes_scaffolding_names OK")
