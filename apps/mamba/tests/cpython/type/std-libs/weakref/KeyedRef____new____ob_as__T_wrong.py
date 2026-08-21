# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "type"
# case = "KeyedRef____new____ob_as__T_wrong"
# subject = "weakref.KeyedRef.__new__(ob: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ob"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/weakref.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed ob
# mamba-strict-type: TypeError
"""Type wall: weakref.KeyedRef.__new__(ob: _T); call it with the wrong type.

typeshed contract: ob is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from weakref import KeyedRef
obj = object.__new__(KeyedRef)
try:
    obj.__new__(_W(), None, None)  # ob: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
