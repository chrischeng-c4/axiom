# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "node_returns_str"
# subject = "platform.node"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.node: node() returns a str hostname (may be empty, so only type is asserted)"""
import platform

out = platform.node()
assert type(out).__name__ == "str", "node() returns str"

print("node_returns_str OK")
