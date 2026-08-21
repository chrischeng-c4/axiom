# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "error_filter_turns_warn_into_raise"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: under simplefilter("error") inside a catch_warnings block, warnings.warn raises the warning category as an exception (UserWarning)"""
import warnings

with warnings.catch_warnings():
    warnings.simplefilter("error")
    _raised = False
    try:
        warnings.warn("turned_into_error", UserWarning)
    except UserWarning:
        _raised = True
    assert _raised, "error filter turns warn into a raise"

print("error_filter_turns_warn_into_raise OK")
