# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "surface"
# case = "reload_is_callable"
# subject = "importlib.reload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.reload: reload_is_callable (surface)."""
import importlib

assert callable(importlib.reload)
print("reload_is_callable OK")
