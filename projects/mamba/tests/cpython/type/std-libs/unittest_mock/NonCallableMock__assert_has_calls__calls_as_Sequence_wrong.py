# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock__assert_has_calls__calls_as_Sequence_wrong"
# subject = "unittest.mock.NonCallableMock.assert_has_calls(calls: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed calls"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed calls
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.assert_has_calls(calls: Sequence); call it with the wrong type.

typeshed contract: calls is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
try:
    obj.assert_has_calls(_W())  # calls: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
