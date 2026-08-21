# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "formatwarning_renders_canonical_line"
# subject = "warnings.formatwarning"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.formatwarning: formatwarning renders '<file>:<lineno>: <Category>: <message>\\n' exactly, e.g. 'app.py:42: UserWarning: disk low\\n'"""
import warnings

line = warnings.formatwarning("disk low", UserWarning, "app.py", 42)
assert line == "app.py:42: UserWarning: disk low\n", f"line = {line!r}"

print("formatwarning_renders_canonical_line OK")
