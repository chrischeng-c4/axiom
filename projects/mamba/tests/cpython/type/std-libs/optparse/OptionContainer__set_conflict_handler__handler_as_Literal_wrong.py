# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__set_conflict_handler__handler_as_Literal_wrong"
# subject = "optparse.OptionContainer.set_conflict_handler(handler: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed handler"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed handler
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.set_conflict_handler(handler: Literal); call it with the wrong type.

typeshed contract: handler is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.set_conflict_handler(_W())  # handler: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
