# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "system_returns_nonempty_str"
# subject = "platform.system"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.system: system() returns a non-empty str naming the OS (value is host-dependent, so only shape is asserted)"""
import platform

out = platform.system()
assert type(out).__name__ == "str", "system() returns str"
assert len(out) > 0, "system() is non-empty"

print("system_returns_nonempty_str OK")
