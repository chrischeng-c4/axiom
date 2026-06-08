# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "lookup_unknown_codec_raises"
# subject = "codecs.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.lookup: lookup_unknown_codec_raises (errors)."""
import codecs

_raised = False
try:
    codecs.lookup('not_a_real_codec_xyz')
except LookupError:
    _raised = True
assert _raised, "lookup_unknown_codec_raises: expected LookupError"
print("lookup_unknown_codec_raises OK")
