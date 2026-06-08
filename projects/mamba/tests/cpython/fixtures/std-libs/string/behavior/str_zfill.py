# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_zfill"
# subject = "str.zfill"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.zfill: zfill left-pads with zeros keeping a leading sign: '42'.zfill(5)=='00042' and '-42'.zfill(5)=='-0042'"""
import builtins  # noqa: F401

assert "42".zfill(5) == "00042", "zfill pads"
assert "-42".zfill(5) == "-0042", "zfill keeps sign"
print("str_zfill OK")
