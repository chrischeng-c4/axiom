# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "functional_unicode_member_name"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: non-Latin (Unicode) identifiers are valid member names via the functional API"""
import enum

greek = enum.Enum("greek", ("alpha_α", "B", "C"))

assert getattr(greek, "alpha_α").value == 1, "Unicode member name is valid"
assert greek.B.value == 2, "subsequent members number from 1"

print("functional_unicode_member_name OK")
