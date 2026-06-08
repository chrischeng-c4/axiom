# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "default_selector_is_callable"
# subject = "selectors.DefaultSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.DefaultSelector: default_selector_is_callable (surface)."""
import selectors

assert callable(selectors.DefaultSelector)
print("default_selector_is_callable OK")
