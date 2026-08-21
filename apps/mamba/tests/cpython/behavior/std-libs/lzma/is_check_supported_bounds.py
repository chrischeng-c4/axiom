# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "is_check_supported_bounds"
# subject = "lzma.is_check_supported"
# kind = "semantic"
# xfail = "lzma.is_check_supported is not implemented (src/runtime/stdlib/lzma_mod.rs)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.is_check_supported: is_check_supported(CHECK_NONE) is True and an id above CHECK_ID_MAX is False"""
import lzma


assert lzma.is_check_supported(lzma.CHECK_NONE) is True, "CHECK_NONE supported"
assert lzma.is_check_supported(lzma.CHECK_ID_MAX + 1) is False, "above max unsupported"
print("is_check_supported_bounds OK")
