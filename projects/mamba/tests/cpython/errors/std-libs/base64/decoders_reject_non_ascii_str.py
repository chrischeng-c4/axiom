# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "errors"
# case = "decoders_reject_non_ascii_str"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64decode: every str-accepting decoder (b64/standard_b64/urlsafe_b64/b32/b16/b85/a85) rejects a str containing non-ASCII characters with ValueError"""
import base64

_decoders = (
    ("b64decode", base64.b64decode),
    ("standard_b64decode", base64.standard_b64decode),
    ("urlsafe_b64decode", base64.urlsafe_b64decode),
    ("b32decode", base64.b32decode),
    ("b16decode", base64.b16decode),
    ("b85decode", base64.b85decode),
    ("a85decode", base64.a85decode),
)
for _name, _fn in _decoders:
    _raised = False
    try:
        _fn("with non-ascii Ë")
    except ValueError:
        _raised = True
    assert _raised, _name + " accepted non-ascii str"
print("decoders_reject_non_ascii_str OK")
