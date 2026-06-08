# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "repeat__stmt_as__Stmt_wrong"
# subject = "timeit.repeat(stmt: _Stmt)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.repeat(stmt: _Stmt); call it with the wrong type.

typeshed contract: stmt is _Stmt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from timeit import repeat
try:
    repeat(_W())  # stmt: _Stmt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
