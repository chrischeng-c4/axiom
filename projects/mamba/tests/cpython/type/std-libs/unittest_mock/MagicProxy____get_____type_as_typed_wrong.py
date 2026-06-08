# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "MagicProxy____get_____type_as_typed_wrong"
# subject = "unittest.mock.MagicProxy.__get__(_type: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed _type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed _type
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.MagicProxy.__get__(_type: typed); call it with the wrong type.

typeshed contract: _type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.mock import MagicProxy
obj = object.__new__(MagicProxy)
try:
    obj.__get__(None, _W())  # _type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
