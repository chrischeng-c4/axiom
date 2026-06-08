# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "legacy_base64_test_case__test_encodebytes"
# subject = "cpython.test_base64.LegacyBase64TestCase.test_encodebytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::LegacyBase64TestCase::test_encodebytes
"""Auto-ported test: LegacyBase64TestCase::test_encodebytes (CPython 3.12 oracle)."""


from array import array
import base64


def check_type_errors(func):
    for value in ("", []):
        try:
            func(value)
        except TypeError:
            pass
        else:
            raise AssertionError(f"expected TypeError for {value!r}")

    multidimensional = memoryview(b"1234").cast("B", (2, 2))
    try:
        func(multidimensional)
    except TypeError:
        pass
    else:
        raise AssertionError("expected TypeError for multidimensional memoryview")

    int_data = memoryview(b"1234").cast("I")
    try:
        func(int_data)
    except TypeError:
        pass
    else:
        raise AssertionError("expected TypeError for non-byte-format memoryview")


assert base64.encodebytes(b"www.python.org") == b"d3d3LnB5dGhvbi5vcmc=\n"
assert base64.encodebytes(b"a") == b"YQ==\n"
assert base64.encodebytes(b"ab") == b"YWI=\n"
assert base64.encodebytes(b"abc") == b"YWJj\n"
assert base64.encodebytes(b"") == b""
assert (
    base64.encodebytes(
        b"abcdefghijklmnopqrstuvwxyz"
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        b"0123456789!@#0^&*();:<>,. []{}"
    )
    == b"YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNE"
    b"RUZHSElKS0xNTk9QUVJTVFVWV1hZWjAxMjM0\nNT"
    b"Y3ODkhQCMwXiYqKCk7Ojw+LC4gW117fQ==\n"
)
assert base64.encodebytes(b"Aladdin:open sesame") == b"QWxhZGRpbjpvcGVuIHNlc2FtZQ==\n"
assert base64.encodebytes(bytearray(b"abc")) == b"YWJj\n"
assert base64.encodebytes(memoryview(b"abc")) == b"YWJj\n"
assert base64.encodebytes(array("B", b"abc")) == b"YWJj\n"
check_type_errors(base64.encodebytes)

print("LegacyBase64TestCase::test_encodebytes: ok")
