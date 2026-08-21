# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "module_action_emits_once"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: the "module" action collapses repeats of the same warning from one module to a single emission across 5 warns"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("module")
    for _ in range(5):
        warnings.warn("same module", UserWarning)
    assert len(recorded) == 1, f"module emits once: {len(recorded)!r}"

print("module_action_emits_once OK")
