# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "console_locals_seed_visible"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: a fresh InteractiveConsole exposes its seeded locals dict via .locals, and the seeded value ('init_val' == 99) is readable"""
import code

_cons = code.InteractiveConsole({"init_val": 99})
assert isinstance(_cons.locals, dict), f"locals type = {type(_cons.locals)!r}"
assert _cons.locals.get("init_val") == 99, \
    f"locals['init_val'] = {_cons.locals.get('init_val')!r}"

print("console_locals_seed_visible OK")
