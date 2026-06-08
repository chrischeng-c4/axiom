# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "once_action_emits_once"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: the "once" action collapses identical repeats globally to a single emission across 5 warns"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("once")
    for _ in range(5):
        warnings.warn("just once", UserWarning)
    assert len(recorded) == 1, f"once emits once: {len(recorded)!r}"

print("once_action_emits_once OK")
