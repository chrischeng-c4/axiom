# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_returns_str"
# subject = "platform.platform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.platform: platform() returns a str summary of the platform (value host-dependent, only type asserted)"""
import platform

out = platform.platform()
assert type(out).__name__ == "str", "platform() returns str"

print("platform_returns_str OK")
