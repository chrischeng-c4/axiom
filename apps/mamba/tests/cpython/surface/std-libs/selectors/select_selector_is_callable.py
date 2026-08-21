# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "select_selector_is_callable"
# subject = "selectors.SelectSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.SelectSelector: select_selector_is_callable (surface)."""
import selectors

assert callable(selectors.SelectSelector)
print("select_selector_is_callable OK")
