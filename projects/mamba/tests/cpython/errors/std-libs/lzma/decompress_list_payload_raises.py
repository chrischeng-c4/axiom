# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "errors"
# case = "decompress_list_payload_raises"
# subject = "lzma.decompress"
# kind = "mechanical"
# xfail = "lzma.decompress accepts a non-bytes payload (returns empty) instead of raising TypeError (src/runtime/stdlib/lzma_mod.rs:132-144)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: decompress_list_payload_raises (errors)."""
import lzma

_raised = False
try:
    lzma.decompress([])
except TypeError:
    _raised = True
assert _raised, "decompress_list_payload_raises: expected TypeError"
print("decompress_list_payload_raises OK")
