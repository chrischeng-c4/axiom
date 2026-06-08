# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "enterModuleContext__cm_as_AbstractContextManager_wrong"
# subject = "unittest.case.enterModuleContext(cm: AbstractContextManager)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cm"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cm
# mamba-strict-type: TypeError
"""Type wall: unittest.case.enterModuleContext(cm: AbstractContextManager); call it with the wrong type.

typeshed contract: cm is AbstractContextManager. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import enterModuleContext
try:
    enterModuleContext(_W())  # cm: AbstractContextManager <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
