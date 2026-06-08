# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getenv_default_when_missing"
# subject = "os.getenv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getenv: os.getenv returns the supplied default for an undefined variable and a str for a present one"""
import os

missing = os.getenv("NONEXISTENT_VAR_XYZ", "default_val")
assert missing == "default_val", f"getenv default = {missing!r}"

# A present variable (PATH is set in every supported environment) reads as str.
present = os.getenv("PATH")
assert present is None or isinstance(present, str), f"PATH type = {type(present)!r}"
print("getenv_default_when_missing OK")
