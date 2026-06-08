# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "formatwarning_appends_source_line"
# subject = "warnings.formatwarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.formatwarning: the optional line= source argument is appended on its own line indented two spaces after the canonical warning line"""
import warnings

line = warnings.formatwarning("old api", DeprecationWarning, "lib.py", 7, line="foo()")
assert line == "lib.py:7: DeprecationWarning: old api\n  foo()\n", f"line = {line!r}"

print("formatwarning_appends_source_line OK")
