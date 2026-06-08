# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "surrogate_strict_encode_raises"
# subject = "str.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: surrogate_strict_encode_raises (errors)."""
import codecs

_raised = False
try:
    '\ud901'.encode('utf-8')
except UnicodeEncodeError:
    _raised = True
assert _raised, "surrogate_strict_encode_raises: expected UnicodeEncodeError"
print("surrogate_strict_encode_raises OK")
