# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "node_is_callable"
# subject = "platform.node"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.node: node_is_callable (surface)."""
import platform

assert callable(platform.node)
print("node_is_callable OK")
