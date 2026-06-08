# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "processor_returns_str"
# subject = "platform.processor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.processor: processor() returns a str (may be empty on some hosts, only type asserted)"""
import platform

out = platform.processor()
assert type(out).__name__ == "str", "processor() returns str"

print("processor_returns_str OK")
