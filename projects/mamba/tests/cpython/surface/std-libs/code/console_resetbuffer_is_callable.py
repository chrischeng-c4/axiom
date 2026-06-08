# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "console_resetbuffer_is_callable"
# subject = "code.InteractiveConsole.resetbuffer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole.resetbuffer: console_resetbuffer_is_callable (surface)."""
import code

assert callable(code.InteractiveConsole.resetbuffer)
print("console_resetbuffer_is_callable OK")
