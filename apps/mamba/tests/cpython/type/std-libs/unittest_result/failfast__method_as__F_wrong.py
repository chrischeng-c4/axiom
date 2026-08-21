# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "failfast__method_as__F_wrong"
# subject = "unittest.result.failfast(method: _F)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed method"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed method
# mamba-strict-type: TypeError
"""Type wall: unittest.result.failfast(method: _F); call it with the wrong type.

typeshed contract: method is _F. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import failfast
try:
    failfast(_W())  # method: _F <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
