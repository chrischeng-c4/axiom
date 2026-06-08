# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "undefined_codec_refuses_everything"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the 'undefined' codec raises UnicodeError for encode and decode of any input, and even non-strict handlers (ignore/replace/backslashreplace) cannot make it succeed"""
import codecs

# The 'undefined' codec refuses to encode or decode anything.
for _call in (
    lambda: codecs.encode("abc", "undefined"),
    lambda: codecs.decode(b"abc", "undefined"),
    lambda: codecs.encode("", "undefined"),
):
    _raised = False
    try:
        _call()
    except UnicodeError:
        _raised = True
    assert _raised, "'undefined' codec should raise UnicodeError"
# Even non-strict handlers cannot make 'undefined' succeed.
for _errors in ("strict", "ignore", "replace", "backslashreplace"):
    _raised = False
    try:
        codecs.encode("abc", "undefined", _errors)
    except UnicodeError:
        _raised = True
    assert _raised, f"'undefined' with {_errors!r} still raises"

print("undefined_codec_refuses_everything OK")
