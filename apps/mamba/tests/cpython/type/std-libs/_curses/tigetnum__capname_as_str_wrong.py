# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses"
# dimension = "type"
# case = "tigetnum__capname_as_str_wrong"
# subject = "_curses.tigetnum(capname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _curses.tigetnum(capname: str); call it with the wrong type.

typeshed contract: capname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _curses import tigetnum
try:
    tigetnum(12345)  # capname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
