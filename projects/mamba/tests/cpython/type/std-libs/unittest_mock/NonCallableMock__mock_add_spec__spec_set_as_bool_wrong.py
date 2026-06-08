# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock__mock_add_spec__spec_set_as_bool_wrong"
# subject = "unittest.mock.NonCallableMock.mock_add_spec(spec_set: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed spec_set"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed spec_set
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.mock_add_spec(spec_set: bool); call it with the wrong type.

typeshed contract: spec_set is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
try:
    obj.mock_add_spec(None, "not_a_bool")  # spec_set: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
