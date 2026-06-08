# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "strenum_custom_str_override"
# subject = "enum.StrEnum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.StrEnum: a custom __str__ from a mixin base overrides StrEnum's __str__ while value equality is preserved"""
import enum

class Loud:
    def __str__(self):
        return "LOUD"

class Shout(Loud, enum.StrEnum):
    HI = "hi"
    __str__ = Loud.__str__

assert Shout.HI == "hi"                 # value equality preserved
assert str(Shout.HI) == "LOUD"          # custom __str__ overrides StrEnum

print("strenum_custom_str_override OK")
