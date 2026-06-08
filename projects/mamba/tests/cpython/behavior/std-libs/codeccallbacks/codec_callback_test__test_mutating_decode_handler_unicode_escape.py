# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_mutating_decode_handler_unicode_escape"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_mutating_decode_handler_unicode_escape"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_mutating_decode_handler_unicode_escape
"""Auto-ported test: CodecCallbackTest::test_mutating_decode_handler_unicode_escape."""


import codecs
import warnings


decode = codecs.unicode_escape_decode
data = {
    br"\x0": (b"\\", 0),
    br"\x3": (b"xxx\\", 3),
    br"\x5": (b"x\\", 1),
}


def mutating(exc):
    if isinstance(exc, UnicodeDecodeError):
        replacement = data.get(exc.object[:exc.end])
        if replacement is not None:
            exc.object = replacement[0] + exc.object[exc.end:]
            return ("\u0404", replacement[1])
    raise AssertionError("don't know how to handle %r" % exc)


codecs.register_error("test.mutating2", mutating)


def check(input_data, expected, msg):
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always", DeprecationWarning)
        assert decode(input_data, "test.mutating2") == (expected, len(input_data))
    messages = [str(item.message) for item in caught if issubclass(item.category, DeprecationWarning)]
    assert any(msg in message for message in messages), messages


check(br"\x0n\z", "\u0404\n\\z", r"invalid escape sequence '\z'")
check(br"\x0n\501", "\u0404\n\u0141", r"invalid octal escape sequence '\501'")
check(br"\x0z", "\u0404\\z", r"invalid escape sequence '\z'")

check(br"\x3n\zr", "\u0404\n\\zr", r"invalid escape sequence '\z'")
check(br"\x3zr", "\u0404\\zr", r"invalid escape sequence '\z'")
check(br"\x3z5", "\u0404\\z5", r"invalid escape sequence '\z'")
check(memoryview(br"\x3z5x")[:-1], "\u0404\\z5", r"invalid escape sequence '\z'")
check(memoryview(br"\x3z5xy")[:-2], "\u0404\\z5", r"invalid escape sequence '\z'")

check(br"\x5n\z", "\u0404\n\\z", r"invalid escape sequence '\z'")
check(br"\x5n\501", "\u0404\n\u0141", r"invalid octal escape sequence '\501'")
check(br"\x5z", "\u0404\\z", r"invalid escape sequence '\z'")
check(memoryview(br"\x5zy")[:-1], "\u0404\\z", r"invalid escape sequence '\z'")

print("CodecCallbackTest::test_mutating_decode_handler_unicode_escape: ok")
