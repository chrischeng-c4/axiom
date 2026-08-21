# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_explicit_default_emits_once_per_location"
# subject = "warnings.warn_explicit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn_explicit: under the "default" action, repeated warn_explicit at the same (message, category, location) with a shared registry emits only once and populates the registry"""
import warnings

# warn_explicit threads its de-dup bookkeeping through the registry argument.
registry = {}
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("default")
    for _ in range(4):
        warnings.warn_explicit("repeat", UserWarning, "f.py", 10, registry=registry)
    assert len(recorded) == 1, f"default emits once: {len(recorded)!r}"
    assert registry, "registry was populated"

print("warn_explicit_default_emits_once_per_location OK")
