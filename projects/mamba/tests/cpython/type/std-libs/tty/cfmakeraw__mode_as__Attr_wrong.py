# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "type"
# case = "cfmakeraw__mode_as__Attr_wrong"
# subject = "tty.cfmakeraw(mode: _Attr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tty.cfmakeraw(mode: _Attr); call it with the wrong type.

typeshed contract: mode is _Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tty import cfmakeraw
try:
    cfmakeraw(_W())  # mode: _Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
