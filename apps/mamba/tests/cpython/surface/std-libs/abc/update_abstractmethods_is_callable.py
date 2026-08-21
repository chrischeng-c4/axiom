# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "update_abstractmethods_is_callable"
# subject = "abc.update_abstractmethods"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.update_abstractmethods: update_abstractmethods_is_callable (surface)."""
import abc

assert callable(abc.update_abstractmethods)
print("update_abstractmethods_is_callable OK")
