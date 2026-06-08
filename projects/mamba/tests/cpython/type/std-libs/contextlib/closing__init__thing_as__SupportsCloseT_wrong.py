# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "closing__init__thing_as__SupportsCloseT_wrong"
# subject = "contextlib.closing.__init__(thing: _SupportsCloseT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.closing.__init__(thing: _SupportsCloseT); call it with the wrong type.

typeshed contract: thing is _SupportsCloseT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import closing
try:
    closing(_W())  # thing: _SupportsCloseT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
