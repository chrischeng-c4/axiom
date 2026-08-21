# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "abstractmethod_is_callable"
# subject = "selectors.abstractmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.abstractmethod: abstractmethod_is_callable (surface)."""
import selectors

assert callable(selectors.abstractmethod)
print("abstractmethod_is_callable OK")
