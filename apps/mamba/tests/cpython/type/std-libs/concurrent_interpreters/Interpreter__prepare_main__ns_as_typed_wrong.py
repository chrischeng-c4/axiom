# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "Interpreter__prepare_main__ns_as_typed_wrong"
# subject = "concurrent.interpreters.Interpreter.prepare_main(ns: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.Interpreter.prepare_main(ns: typed); call it with the wrong type.

typeshed contract: ns is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters import Interpreter
obj = object.__new__(Interpreter)
try:
    obj.prepare_main(_W())  # ns: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
