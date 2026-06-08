# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_locals_shared_with_engine"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: the locals dict passed to InteractiveConsole is shared: a pushed statement reading a seeded name ('start') and writing a new one ('result') mutates that very dict"""
import code

_shared = {"start": 0}
_cons = code.InteractiveConsole(_shared)
_cons.push("result = start + 100")
assert _shared.get("result") == 100, f"shared locals: {_shared.get('result')!r}"

print("console_locals_shared_with_engine OK")
