# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abstractstaticmethod_is_callable"
# subject = "abc.abstractstaticmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.abstractstaticmethod: abstractstaticmethod_is_callable (surface)."""
import abc

assert callable(abc.abstractstaticmethod)
print("abstractstaticmethod_is_callable OK")
