# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock__attach_mock__mock_as_NonCallableMock_wrong"
# subject = "unittest.mock.NonCallableMock.attach_mock(mock: NonCallableMock)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.attach_mock(mock: NonCallableMock); call it with the wrong type.

typeshed contract: mock is NonCallableMock. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
try:
    obj.attach_mock(_W(), "")  # mock: NonCallableMock <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
