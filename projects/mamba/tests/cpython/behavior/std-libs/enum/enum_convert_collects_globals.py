# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "enum_convert_collects_globals"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: Enum._convert_ scans module globals and builds plain enum members with aliases."""
import enum

CONVERT_ENUM_A = "alpha"
CONVERT_ENUM_B = "beta"
CONVERT_ENUM_C = "beta"  # same value as B -> alias

EnumCfg = enum.Enum._convert_(
    "EnumCfg", __name__, filter=lambda name: name.startswith("CONVERT_ENUM_"))
assert EnumCfg.CONVERT_ENUM_A.name == "CONVERT_ENUM_A"
assert EnumCfg.CONVERT_ENUM_A.value == "alpha"
assert EnumCfg.CONVERT_ENUM_B.value == "beta"
assert EnumCfg.CONVERT_ENUM_C is EnumCfg.CONVERT_ENUM_B
assert list(EnumCfg) == [EnumCfg.CONVERT_ENUM_A, EnumCfg.CONVERT_ENUM_B]

print("enum_convert_collects_globals OK")
