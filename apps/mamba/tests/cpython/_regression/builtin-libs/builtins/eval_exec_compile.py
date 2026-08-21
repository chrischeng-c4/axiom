# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins: eval / exec / compile happy paths and namespace handling."""

# Distilled from CPython Lib/test/test_builtin.py test_eval/test_exec/
# test_compile (re-curated: dropped BOM, top-level-await, ast/optimize
# C-flag internals; kept the portable namespace + mode contract).

# eval evaluates an expression and returns its value.
assert eval("1 + 1") == 2
assert eval(" 1 + 1\n") == 2

# eval reads names from the supplied globals and locals dicts; locals win.
g = {"a": 1, "b": 2}
ldict = {"b": 200, "c": 300}
assert eval("a", g, ldict) == 1     # from globals
assert eval("b", g, ldict) == 200   # locals shadows globals
assert eval("c", g, ldict) == 300   # from locals

# exec runs statements and mutates the supplied namespace.
ns = {}
exec("z = 1 + 1", ns)
assert ns["z"] == 2

# exec with separate globals/locals writes assignments into locals.
gl, lo = {}, {}
exec("p = 10\nq = p * 2", gl, lo)
assert lo["p"] == 10
assert lo["q"] == 20

# compile produces a reusable code object for exec mode.
code = compile("x = 5\ny = x * 3", "<gen>", "exec")
ns2 = {}
exec(code, ns2)
assert ns2["x"] == 5
assert ns2["y"] == 15

# compile in eval mode yields an expression code object.
expr = compile("7 * 6", "<gen>", "eval")
assert eval(expr) == 42

# optimize=2 strips docstrings from compiled code.
src = 'def f():\n    "the doc"\n    return f.__doc__\n'
keep = {}
exec(compile(src, "<o>", "exec", optimize=0), keep)
assert keep["f"]() == "the doc"
strip = {}
exec(compile(src, "<o>", "exec", optimize=2), strip)
assert strip["f"]() is None

# An invalid compile mode raises ValueError.
try:
    compile("pass", "<m>", "badmode")
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# eval of a syntactically invalid expression raises SyntaxError.
try:
    eval("1 +")
    raise AssertionError("expected SyntaxError")
except SyntaxError:
    pass

print("eval_exec_compile OK")
