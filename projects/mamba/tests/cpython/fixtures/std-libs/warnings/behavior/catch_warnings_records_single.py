# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "catch_warnings_records_single"
# subject = "warnings.catch_warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings(record=True) with simplefilter("always") captures one WarningMessage carrying the category and message text"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("captured warning", UserWarning)
    assert len(recorded) == 1, f"captured = {len(recorded)!r}"
    assert issubclass(recorded[0].category, UserWarning), f"category = {recorded[0].category!r}"
    assert "captured warning" in str(recorded[0].message), f"message = {str(recorded[0].message)!r}"

print("catch_warnings_records_single OK")
