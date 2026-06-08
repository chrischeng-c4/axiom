# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "runsource_updates_locals"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: InteractiveInterpreter.runsource executes complete source and the result lands in the supplied locals dict: 'answer = 6 * 7' sets answer == 42"""
import code

_local = {}
_interp = code.InteractiveInterpreter(_local)
_interp.runsource("answer = 6 * 7")
assert _local.get("answer") == 42, f"runsource set local: {_local.get('answer')!r}"

print("runsource_updates_locals OK")
