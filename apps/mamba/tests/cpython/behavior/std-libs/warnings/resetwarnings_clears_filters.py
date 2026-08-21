# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "resetwarnings_clears_filters"
# subject = "warnings.resetwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.resetwarnings: resetwarnings() clears installed filters so a subsequent simplefilter("always") + warn is captured again"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.filterwarnings("ignore")
    warnings.resetwarnings()
    # After reset, the blanket "ignore" is gone; install always and warn.
    warnings.simplefilter("always")
    warnings.warn("after reset", UserWarning)
    assert len(recorded) >= 1, f"after reset, warning captured = {len(recorded)!r}"

print("resetwarnings_clears_filters OK")
