# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "always_captures_every_occurrence"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter("always") captures every occurrence: warning the same message 5 times records 5 messages"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    for _i in range(5):
        warnings.warn("repeat", UserWarning)
    assert len(recorded) == 5, f"always captures all: {len(recorded)!r}"

print("always_captures_every_occurrence OK")
