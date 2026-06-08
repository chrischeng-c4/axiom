# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_defaults_to_userwarning"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: a bare-string warn() with no category defaults the recorded category to UserWarning"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("no category given")
    assert recorded[0].category is UserWarning, f"default = {recorded[0].category!r}"

print("warn_defaults_to_userwarning OK")
