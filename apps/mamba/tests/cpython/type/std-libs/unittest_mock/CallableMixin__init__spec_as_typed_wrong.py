# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "CallableMixin__init__spec_as_typed_wrong"
# subject = "unittest.mock.CallableMixin.__init__(spec: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed spec"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed spec
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.CallableMixin.__init__(spec: typed); call it with the wrong type.

typeshed contract: spec is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.mock import CallableMixin
try:
    CallableMixin(_W())  # spec: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
