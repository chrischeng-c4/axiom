# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_explicit_distinct_locations_each_emit"
# subject = "warnings.warn_explicit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn_explicit: distinct linenos are distinct registry keys under "default", so warn_explicit at two different locations emits twice"""
import warnings

registry = {}
with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("default")
    warnings.warn_explicit("loc", UserWarning, "f.py", 1, registry=registry)
    warnings.warn_explicit("loc", UserWarning, "f.py", 2, registry=registry)
    assert len(recorded) == 2, f"distinct locations both emit: {len(recorded)!r}"

print("warn_explicit_distinct_locations_each_emit OK")
