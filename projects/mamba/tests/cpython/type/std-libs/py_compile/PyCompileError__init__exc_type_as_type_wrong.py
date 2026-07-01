# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "py_compile"
# dimension = "type"
# case = "PyCompileError__init__exc_type_as_type_wrong"
# subject = "py_compile.PyCompileError.__init__(exc_type: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/py_compile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: py_compile.PyCompileError.__init__(exc_type: type); call it with the wrong type.

typeshed contract: exc_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from py_compile import PyCompileError
try:
    PyCompileError(_W(), None, "")  # exc_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
