# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "release_returns_str"
# subject = "platform.release"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.release: release() returns a str OS release (value host-dependent, only type asserted)"""
import platform

out = platform.release()
assert type(out).__name__ == "str", "release() returns str"

print("release_returns_str OK")
