# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "bin_ascii_test__test_base64_strict_mode"
# subject = "cpython.test_binascii.BinASCIITest.test_base64_strict_mode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binascii.py::BinASCIITest::test_base64_strict_mode
"""Auto-ported test: BinASCIITest::test_base64_strict_mode (CPython 3.12 oracle)."""


import binascii
import re


def assert_raises_regex(exc_type, pattern, fn, *args, **kwargs):
    try:
        fn(*args, **kwargs)
    except exc_type as exc:
        assert re.search(pattern, str(exc)), (pattern, str(exc))
        return
    raise AssertionError(f"expected {exc_type.__name__}")


def assert_regex_template(assert_regex, data, non_strict_mode_expected_result):
    assert_raises_regex(
        binascii.Error,
        assert_regex,
        binascii.a2b_base64,
        bytes(data),
        strict_mode=True,
    )
    assert (
        binascii.a2b_base64(bytes(data), strict_mode=False)
        == non_strict_mode_expected_result
    )
    assert binascii.a2b_base64(bytes(data)) == non_strict_mode_expected_result


def assert_excess_data(data, non_strict_mode_expected_result):
    assert_regex_template(r"(?i)Excess data", data, non_strict_mode_expected_result)


def assert_non_base64_data(data, non_strict_mode_expected_result):
    assert_regex_template(
        r"(?i)Only base64 data", data, non_strict_mode_expected_result
    )


def assert_leading_padding(data, non_strict_mode_expected_result):
    assert_regex_template(r"(?i)Leading padding", data, non_strict_mode_expected_result)


def assert_discontinuous_padding(data, non_strict_mode_expected_result):
    assert_regex_template(
        r"(?i)Discontinuous padding", data, non_strict_mode_expected_result
    )


def assert_excess_padding(data, non_strict_mode_expected_result):
    assert_regex_template(r"(?i)Excess padding", data, non_strict_mode_expected_result)


assert_excess_data(b"ab==a", b"i")
assert_excess_data(b"ab===", b"i")
assert_excess_data(b"ab====", b"i")
assert_excess_data(b"ab==:", b"i")
assert_excess_data(b"abc=a", b"i\xb7")
assert_excess_data(b"abc=:", b"i\xb7")
assert_excess_data(b"ab==\n", b"i")
assert_excess_data(b"abc==", b"i\xb7")
assert_excess_data(b"abc===", b"i\xb7")
assert_excess_data(b"abc====", b"i\xb7")
assert_excess_data(b"abc=====", b"i\xb7")

assert_non_base64_data(b"\nab==", b"i")
assert_non_base64_data(b"ab:(){:|:&};:==", b"i")
assert_non_base64_data(b"a\nb==", b"i")
assert_non_base64_data(b"a\x00b==", b"i")

assert_leading_padding(b"=", b"")
assert_leading_padding(b"==", b"")
assert_leading_padding(b"===", b"")
assert_leading_padding(b"====", b"")
assert_leading_padding(b"=====", b"")
assert_discontinuous_padding(b"ab=c=", b"i\xb7")
assert_discontinuous_padding(b"ab=ab==", b"i\xb6\x9b")
assert_excess_padding(b"abcd=", b"i\xb7\x1d")
assert_excess_padding(b"abcd==", b"i\xb7\x1d")
assert_excess_padding(b"abcd===", b"i\xb7\x1d")
assert_excess_padding(b"abcd====", b"i\xb7\x1d")
assert_excess_padding(b"abcd=====", b"i\xb7\x1d")

print("BinASCIITest::test_base64_strict_mode: ok")
