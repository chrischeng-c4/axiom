# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "missing_hook_resolves_lookup"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a _missing_ classmethod resolves an otherwise-invalid lookup to a member; when it returns None the original ValueError propagates with no chained context"""
import enum


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3

    @classmethod
    def _missing_(cls, value):
        if value == "three":
            return cls.BLUE
        return None


# _missing_ maps an alternate key to a real member.
assert Color("three") is Color.BLUE, "_missing_ resolves to a member"

# When _missing_ returns None, the original ValueError propagates with no
# chained context.
_raised = False
try:
    Color(99)
except ValueError as e:
    _raised = True
    assert e.__context__ is None, f"unexpected chained context: {e.__context__!r}"
assert _raised, "unknown value still raises ValueError"

print("missing_hook_resolves_lookup OK")
