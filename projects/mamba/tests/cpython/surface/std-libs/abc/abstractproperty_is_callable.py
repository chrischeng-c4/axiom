# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abstractproperty_is_callable"
# subject = "abc.abstractproperty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.abstractproperty: abstractproperty_is_callable (surface)."""
import abc

assert callable(abc.abstractproperty)
print("abstractproperty_is_callable OK")
