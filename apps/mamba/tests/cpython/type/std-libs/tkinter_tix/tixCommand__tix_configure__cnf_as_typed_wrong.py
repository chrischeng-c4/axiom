# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_tix"
# dimension = "type"
# case = "tixCommand__tix_configure__cnf_as_typed_wrong"
# subject = "tkinter.tix.tixCommand.tix_configure(cnf: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cnf"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/tix.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cnf
# mamba-strict-type: TypeError
"""Type wall: tkinter.tix.tixCommand.tix_configure(cnf: typed); call it with the wrong type.

typeshed contract: cnf is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.tix import tixCommand
obj = object.__new__(tixCommand)
try:
    obj.tix_configure(_W())  # cnf: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
