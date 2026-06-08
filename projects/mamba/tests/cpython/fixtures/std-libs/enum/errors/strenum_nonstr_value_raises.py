# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "strenum_nonstr_value_raises"
# subject = "enum.StrEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.StrEnum: defining a StrEnum with a non-string member value raises TypeError (StrEnum members must be str)"""
import enum


# A StrEnum member value that is not a str is rejected at class creation.
_raised = False
try:
    class BadStr(enum.StrEnum):
        ONE = 1  # not a string
except TypeError:
    _raised = True
assert _raised, "StrEnum with a non-string value must raise TypeError"


# A well-formed StrEnum is accepted and its members are real strings.
class Label(enum.StrEnum):
    ONE = "1"
    TWO = "2"


assert Label.ONE == "1", "well-formed StrEnum member equals its str value"

print("strenum_nonstr_value_raises OK")
