# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "strenum_member_is_str"
# subject = "enum.StrEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.StrEnum: a StrEnum member IS a str: == its value, str() and format() yield the value, and repr is '<Class.NAME: 'value'>'"""
import enum

class Label(enum.StrEnum):
    ONE = "1"
    TWO = "2"

assert Label.ONE == "1"
assert str(Label.ONE) == "1"
assert "{}".format(Label.ONE) == "1"
assert repr(Label.ONE) == "<Label.ONE: '1'>"
assert isinstance(Label.ONE, str)

print("strenum_member_is_str OK")
