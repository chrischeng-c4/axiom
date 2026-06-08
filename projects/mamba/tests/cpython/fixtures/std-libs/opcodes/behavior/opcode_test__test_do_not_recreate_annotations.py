# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcodes"
# dimension = "behavior"
# case = "opcode_test__test_do_not_recreate_annotations"
# subject = "cpython.test_opcodes.OpcodeTest.test_do_not_recreate_annotations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcodes.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcodes.py::OpcodeTest::test_do_not_recreate_annotations
"""Auto-ported test: OpcodeTest::test_do_not_recreate_annotations."""


from test import support


with support.swap_item(globals(), "__annotations__", {}):
    del globals()["__annotations__"]

    class C:
        del __annotations__
        try:
            x: int
        except NameError:
            pass
        else:
            raise AssertionError("expected NameError")


print("OpcodeTest::test_do_not_recreate_annotations: ok")
