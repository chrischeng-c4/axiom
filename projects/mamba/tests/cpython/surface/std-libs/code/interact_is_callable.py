# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "interact_is_callable"
# subject = "code.interact"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.interact: interact_is_callable (surface)."""
import code

assert callable(code.interact)
print("interact_is_callable OK")
