# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "suppress_catches_listed_exceptions_only"
# subject = "contextlib.suppress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.suppress: suppress(KeyError, IndexError) swallows a listed exception but lets an unlisted ValueError propagate out of the with-block"""
import contextlib

# A listed exception is swallowed.
with contextlib.suppress(KeyError, IndexError):
    raise KeyError("suppressed")

# An unlisted exception propagates.
_propagated = False
try:
    with contextlib.suppress(KeyError):
        raise ValueError("not suppressed")
except ValueError:
    _propagated = True
assert _propagated, "an unlisted exception must propagate"

print("suppress_catches_listed_exceptions_only OK")
