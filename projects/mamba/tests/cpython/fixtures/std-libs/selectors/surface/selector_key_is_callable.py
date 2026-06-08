# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "selector_key_is_callable"
# subject = "selectors.SelectorKey"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.SelectorKey: selector_key_is_callable (surface)."""
import selectors

assert callable(selectors.SelectorKey)
print("selector_key_is_callable OK")
