# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "catch_warnings_records_multiple"
# subject = "warnings.catch_warnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings(record=True) collects every emitted warning in order; two warns of different categories yield two records whose categories match"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("first", UserWarning)
    warnings.warn("second", DeprecationWarning)
    assert len(recorded) == 2, f"two warnings = {len(recorded)!r}"
    assert issubclass(recorded[0].category, UserWarning), "first is UserWarning"
    assert issubclass(recorded[1].category, DeprecationWarning), "second is DeprecationWarning"

print("catch_warnings_records_multiple OK")
