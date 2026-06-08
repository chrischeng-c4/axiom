# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "strenum_convert_collects_globals"
# subject = "enum.StrEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.StrEnum: StrEnum._convert_ gathers matching string globals into a StrEnum whose members equal and str() to their string value"""
import enum

CONVERT_STR_HELLO = "hello"
CONVERT_STR_BYE = "goodbye"

StrCfg = enum.StrEnum._convert_(
    "StrCfg", __name__, filter=lambda name: name.startswith("CONVERT_STR_"))
assert StrCfg.CONVERT_STR_HELLO == "hello"
assert StrCfg.CONVERT_STR_BYE == "goodbye"
assert str(StrCfg.CONVERT_STR_BYE) == "goodbye"   # StrEnum str() is the value
assert isinstance(StrCfg.CONVERT_STR_HELLO, str)

print("strenum_convert_collects_globals OK")
