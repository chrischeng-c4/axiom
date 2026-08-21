# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "ignore_filter_suppresses"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter("ignore") suppresses warnings so a record=True catch_warnings collects nothing"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("ignore")
    warnings.warn("ignored", UserWarning)
    assert len(recorded) == 0, f"ignored = {len(recorded)!r}"

print("ignore_filter_suppresses OK")
