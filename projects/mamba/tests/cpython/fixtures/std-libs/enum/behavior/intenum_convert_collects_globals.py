# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "intenum_convert_collects_globals"
# subject = "enum.IntEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntEnum: IntEnum._convert_ scans a module for globals matching a name filter and builds an IntEnum; equal values collapse to aliases"""
import enum

CONVERT_INT_A = 1
CONVERT_INT_B = 2
CONVERT_INT_C = 2  # same value as B -> alias

IntCfg = enum.IntEnum._convert_(
    "IntCfg", __name__, filter=lambda name: name.startswith("CONVERT_INT_"))
assert IntCfg.CONVERT_INT_A == 1
assert IntCfg.CONVERT_INT_B == 2
assert IntCfg.CONVERT_INT_C is IntCfg.CONVERT_INT_B   # duplicate value -> alias
assert len(list(IntCfg)) == 2                          # aliases excluded

print("intenum_convert_collects_globals OK")
