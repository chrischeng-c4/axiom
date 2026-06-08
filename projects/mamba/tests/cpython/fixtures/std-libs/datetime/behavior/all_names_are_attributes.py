# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "behavior"
# case = "all_names_are_attributes"
# subject = "datetime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: every name in datetime.__all__ is a real module attribute"""
import datetime

for name in datetime.__all__:
    assert hasattr(datetime, name), f"__all__ name missing: {name!r}"
print("all_names_are_attributes OK")
