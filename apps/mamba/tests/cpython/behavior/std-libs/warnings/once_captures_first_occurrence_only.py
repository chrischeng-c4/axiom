# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "once_captures_first_occurrence_only"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter("once") collapses identical repeats: warning the same message 5 times records exactly 1"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("once")
    for _i in range(5):
        warnings.warn("once_test", UserWarning)
    assert len(recorded) == 1, f"once captures one: {len(recorded)!r}"

print("once_captures_first_occurrence_only OK")
