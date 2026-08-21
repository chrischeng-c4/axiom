# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abstractmethod_is_callable"
# subject = "abc.abstractmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.abstractmethod: abstractmethod_is_callable (surface)."""
import abc

assert callable(abc.abstractmethod)
print("abstractmethod_is_callable OK")
