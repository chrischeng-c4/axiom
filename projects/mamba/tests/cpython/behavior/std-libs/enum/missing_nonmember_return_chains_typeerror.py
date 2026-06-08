# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "missing_nonmember_return_chains_typeerror"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a _missing_ that returns a non-member raises TypeError chained (via __context__) from the underlying ValueError"""
import enum


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3

    @classmethod
    def _missing_(cls, value):
        if value == "bad":
            return 5  # not a member -> TypeError, chained from a ValueError
        return None


# A _missing_ returning a non-member raises TypeError chained from ValueError.
_raised = False
try:
    Color("bad")
except TypeError as e:
    _raised = True
    assert isinstance(e.__context__, ValueError), f"context = {e.__context__!r}"
assert _raised, "non-member return must raise TypeError"

print("missing_nonmember_return_chains_typeerror OK")
