# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "Interpreter__exec__code_as_typed_wrong"
# subject = "concurrent.interpreters.Interpreter.exec(code: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed code"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed code
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.Interpreter.exec(code: typed); call it with the wrong type.

typeshed contract: code is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters import Interpreter
obj = object.__new__(Interpreter)
try:
    obj.exec(_W())  # code: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
