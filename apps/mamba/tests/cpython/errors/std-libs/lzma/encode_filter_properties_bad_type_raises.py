# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "encode_filter_properties_bad_type_raises"
# subject = "lzma._encode_filter_properties"
# kind = "mechanical"
# xfail = "lzma._encode_filter_properties is not implemented (src/runtime/stdlib/lzma_mod.rs)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma._encode_filter_properties: encode_filter_properties_bad_type_raises (errors)."""
import lzma

_raised = False
try:
    lzma._encode_filter_properties(b'not a dict')
except TypeError:
    _raised = True
assert _raised, "encode_filter_properties_bad_type_raises: expected TypeError"
print("encode_filter_properties_bad_type_raises OK")
