# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_base64errors"
# subject = "cpython.test_binascii.BinASCIITest.test_base64errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_base64errors
"""Auto-ported test: BinASCIITest::test_base64errors (CPython 3.12 oracle)."""


import binascii
import re


def assert_raises_regex(exc_type, pattern, fn, *args, **kwargs):
    try:
        fn(*args, **kwargs)
    except exc_type as exc:
        assert re.search(pattern, str(exc)), (pattern, str(exc))
        return
    raise AssertionError(f"expected {exc_type.__name__}")


def assert_incorrect_padding(data):
    assert_raises_regex(
        binascii.Error,
        r"(?i)Incorrect padding",
        binascii.a2b_base64,
        bytes(data),
    )


assert_incorrect_padding(b"ab")
assert_incorrect_padding(b"ab=")
assert_incorrect_padding(b"abc")
assert_incorrect_padding(b"abcdef")
assert_incorrect_padding(b"abcdef=")
assert_incorrect_padding(b"abcdefg")
assert_incorrect_padding(b"a=b=")
assert_incorrect_padding(b"a\nb=")


def assert_invalid_length(data):
    n_data_chars = len(re.sub(br"[^A-Za-z0-9/+]", b"", data))
    expected_errmsg_re = r"(?i)Invalid.+number of data characters.+" + str(
        n_data_chars
    )
    assert_raises_regex(
        binascii.Error,
        expected_errmsg_re,
        binascii.a2b_base64,
        bytes(data),
    )


assert_invalid_length(b"a")
assert_invalid_length(b"a=")
assert_invalid_length(b"a==")
assert_invalid_length(b"a===")
assert_invalid_length(b"a" * 5)
assert_invalid_length(b"a" * (4 * 87 + 1))
assert_invalid_length(b"A\tB\nC ??DE")

print("BinASCIITest::test_base64errors: ok")
