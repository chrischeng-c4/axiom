# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "poll_selector_is_callable"
# subject = "selectors.PollSelector"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors.PollSelector: poll_selector_is_callable (surface)."""
import selectors

assert callable(selectors.PollSelector)
print("poll_selector_is_callable OK")
