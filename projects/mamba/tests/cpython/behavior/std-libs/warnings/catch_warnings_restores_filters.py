# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "catch_warnings_restores_filters"
# subject = "warnings.catch_warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings restores the filter state on exit: an "error" filter active inside the block is gone afterward, so a later warn no longer raises"""
import warnings

# Inside: an "error" filter is active and warn() raises.
with warnings.catch_warnings():
    warnings.simplefilter("error")
    _raised = False
    try:
        warnings.warn("inside", UserWarning)
    except UserWarning:
        _raised = True
    assert _raised, "error filter active inside block"

# Outside: the error filter is gone, so a fresh always-filter just records.
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("outside", UserWarning)
    assert len(recorded) == 1, f"normal filter after catch_warnings = {len(recorded)!r}"

print("catch_warnings_restores_filters OK")
