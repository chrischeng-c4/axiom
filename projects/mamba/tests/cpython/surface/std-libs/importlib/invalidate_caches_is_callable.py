# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "invalidate_caches_is_callable"
# subject = "importlib.invalidate_caches"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.invalidate_caches: invalidate_caches_is_callable (surface)."""
import importlib

assert callable(importlib.invalidate_caches)
print("invalidate_caches_is_callable OK")
