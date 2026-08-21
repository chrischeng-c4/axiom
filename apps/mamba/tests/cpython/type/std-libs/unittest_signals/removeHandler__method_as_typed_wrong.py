# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_signals"
# dimension = "type"
# case = "removeHandler__method_as_typed_wrong"
# subject = "unittest.signals.removeHandler(method: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed method"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/signals.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed method
# mamba-strict-type: TypeError
"""Type wall: unittest.signals.removeHandler(method: typed); call it with the wrong type.

typeshed contract: method is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.signals import removeHandler
try:
    removeHandler(_W())  # method: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
