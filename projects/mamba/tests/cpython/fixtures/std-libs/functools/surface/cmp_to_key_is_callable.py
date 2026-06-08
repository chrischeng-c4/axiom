# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "cmp_to_key_is_callable"
# subject = "functools.cmp_to_key"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.cmp_to_key: cmp_to_key_is_callable (surface)."""
import functools

assert callable(functools.cmp_to_key)
print("cmp_to_key_is_callable OK")
