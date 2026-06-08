# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getclosurevars_reports_nonlocals_and_builtins"
# subject = "inspect.getclosurevars"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclosurevars: getclosurevars() reports nonlocal and builtin names referenced by a closure; an empty closure yields all-empty ClosureVars"""
import inspect

def make_closure():
    captured = 0

    def inner():
        return len(str(captured))

    return inner

cv = inspect.getclosurevars(make_closure())
assert cv.nonlocals == {"captured": 0}, f"nonlocals = {cv.nonlocals!r}"
assert "len" in cv.builtins, f"builtins = {cv.builtins!r}"

# Empty closure -> all-empty ClosureVars.
empty = inspect.ClosureVars({}, {}, {}, set())
assert inspect.getclosurevars(lambda: True) == empty, "empty closure"

print("getclosurevars_reports_nonlocals_and_builtins OK")
