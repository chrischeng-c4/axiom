# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/decorator_full: decorator grammar + error paths (CPython 3.12 oracle)."""

# PEP 614 (3.9+) relaxed the decorator grammar: any expression is allowed
# after `@`. These all compile cleanly even though they are not plain names.
for expr in (
    "(x,)", "(x, y)", "(x := y)", "(x @ y)", "x[0]",
    "w[x].y.z", "w + x - (y + z)", "x(y)()(z)", "[w, x, y][z]", "x.y",
):
    compile("@%s\ndef f(): pass" % expr, "deco", "exec")
print("relaxed_grammar: ok")


# Statements (not expressions) after `@` are still a SyntaxError.
for stmt in ("x, y", "x = y", "pass", "import sys"):
    try:
        compile("@%s\ndef f(): pass" % stmt, "deco", "exec")
        print("syntax(%r): no_raise" % stmt)
    except SyntaxError:
        pass
print("statement_decorator: SyntaxError")


# A decorator expression that evaluates to a bad value/name raises at the
# moment the function is defined, with the natural exception for the failure.
def unimp(func):
    raise NotImplementedError


cases = [
    ("undef", NameError),            # name not defined
    ("nullval", TypeError),          # None is not callable
    ("nullval.attr", AttributeError),  # attribute on None
    ("unimp", NotImplementedError),  # decorator body raises
]
for expr, exc in cases:
    src = "@%s\ndef f(): pass" % expr
    ns = {"nullval": None, "unimp": unimp}
    try:
        exec(src, ns)
        print("apply(%r): no_raise" % expr)
    except exc as e:
        print("apply(%r):" % expr, type(e).__name__)


# A decorator factory called with the wrong arity raises TypeError when the
# factory itself is invoked, before any wrapping happens.
def needs_one(n):
    def deco(func):
        return func
    return deco


try:
    @needs_one()        # type: ignore[call-arg]
    def g():
        return 1
    print("factory_arity: no_raise")
except TypeError as e:
    print("factory_arity:", type(e).__name__)
