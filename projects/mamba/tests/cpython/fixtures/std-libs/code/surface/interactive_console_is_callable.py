# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "surface"
# case = "interactive_console_is_callable"
# subject = "code.InteractiveConsole"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: interactive_console_is_callable (surface)."""
import code

assert callable(code.InteractiveConsole)
print("interactive_console_is_callable OK")
