# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "open_is_callable"
# subject = "lzma.open"
# kind = "mechanical"
# xfail = "lzma.open is a sentinel-string stub, not callable (src/runtime/stdlib/lzma_mod.rs:81-82)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""lzma.open: open_is_callable (surface)."""
import lzma

assert callable(lzma.open)
print("open_is_callable OK")
