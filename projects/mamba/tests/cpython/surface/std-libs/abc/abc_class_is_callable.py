# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abc_class_is_callable"
# subject = "abc.ABC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.ABC: abc_class_is_callable (surface)."""
import abc

assert callable(abc.ABC)
print("abc_class_is_callable OK")
