# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses"
# dimension = "type"
# case = "halfdelay__tenths_as_int_wrong"
# subject = "_curses.halfdelay(tenths: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _curses.halfdelay(tenths: int); call it with the wrong type.

typeshed contract: tenths is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _curses import halfdelay
try:
    halfdelay("not_an_int")  # tenths: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
