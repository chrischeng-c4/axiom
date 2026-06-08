# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "kqueue_selector_is_callable"
# subject = "selectors.KqueueSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.KqueueSelector: kqueue_selector_is_callable (surface)."""
import selectors

assert callable(selectors.KqueueSelector)
print("kqueue_selector_is_callable OK")
