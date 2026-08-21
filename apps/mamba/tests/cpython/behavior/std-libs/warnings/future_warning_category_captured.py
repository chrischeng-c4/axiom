# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "future_warning_category_captured"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn(msg, FutureWarning) records a WarningMessage whose category is a FutureWarning subclass"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("msg", FutureWarning)
    assert issubclass(recorded[0].category, FutureWarning), "FutureWarning captured"

print("future_warning_category_captured OK")
