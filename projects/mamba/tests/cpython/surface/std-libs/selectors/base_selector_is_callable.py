# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "base_selector_is_callable"
# subject = "selectors.BaseSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.BaseSelector: base_selector_is_callable (surface)."""
import selectors

assert callable(selectors.BaseSelector)
print("base_selector_is_callable OK")
