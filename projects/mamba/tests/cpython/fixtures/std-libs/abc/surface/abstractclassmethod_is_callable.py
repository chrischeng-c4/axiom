# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abstractclassmethod_is_callable"
# subject = "abc.abstractclassmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.abstractclassmethod: abstractclassmethod_is_callable (surface)."""
import abc

assert callable(abc.abstractclassmethod)
print("abstractclassmethod_is_callable OK")
