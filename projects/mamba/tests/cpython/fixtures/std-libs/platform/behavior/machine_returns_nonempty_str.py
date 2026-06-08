# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "machine_returns_nonempty_str"
# subject = "platform.machine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.machine: machine() returns a non-empty str naming the host architecture (value host-dependent, only shape asserted)"""
import platform

out = platform.machine()
assert type(out).__name__ == "str", "machine() returns str"
assert len(out) > 0, "machine() is non-empty"

print("machine_returns_nonempty_str OK")
