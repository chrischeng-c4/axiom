# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_instance_infers_category"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn(DeprecationWarning("...")) infers the category from the instance and keeps the instance as the recorded message object"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn(DeprecationWarning("from instance"))
    assert recorded[0].category is DeprecationWarning, f"inferred = {recorded[0].category!r}"
    assert isinstance(recorded[0].message, DeprecationWarning), "message is the instance"

print("warn_instance_infers_category OK")
