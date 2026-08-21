# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "namedtuple_is_callable"
# subject = "selectors.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.namedtuple: namedtuple_is_callable (surface)."""
import selectors

assert callable(selectors.namedtuple)
print("namedtuple_is_callable OK")
