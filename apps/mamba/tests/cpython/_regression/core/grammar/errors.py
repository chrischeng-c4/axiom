# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: language-level error paths (CPython 3.12 oracle).

Catch-all error coverage: mixed-type ops, missing keys, missing
attributes, out-of-range index -- the dominant TypeError / KeyError /
IndexError surface for language-level operations -- plus grammar/syntax
errors distilled from CPython TestSpecifics + SyntaxTestCase.
"""


# Mixed-type op raises TypeError.
try:
    _ = 1 + "a"  # type: ignore[operator]
    print("mixed: no_raise")
except TypeError as e:
    print("mixed:", type(e).__name__, str(e)[:60])


# Out-of-range index raises IndexError.
try:
    [1, 2][5]
    print("oor: no_raise")
except IndexError as e:
    print("oor:", type(e).__name__, str(e)[:60])


# Missing dict key raises KeyError.
try:
    {}["missing"]
    print("missing_key: no_raise")
except KeyError as e:
    print("missing_key:", type(e).__name__, str(e)[:60])


# Hashing an unhashable raises TypeError.
try:
    hash([1, 2])  # type: ignore[arg-type]
    print("unhashable: no_raise")
except TypeError as e:
    print("unhashable:", type(e).__name__, str(e)[:60])


# --- Grammar / syntax errors (distilled from CPython compile-time tests) ---

# A non-default parameter may not follow a default one.
try:
    exec("def f(a=1, b): pass")
    print("argorder: no_raise")
except SyntaxError as e:
    print("argorder:", type(e).__name__)


# A name cannot be both global and a parameter in the same function.
try:
    exec("def f(a):\n global a\n a = 1")
    print("dupglobal: no_raise")
except SyntaxError as e:
    print("dupglobal:", type(e).__name__)


# None / __debug__ are keywords and cannot be assignment targets.
for stmt in ("None = 0", "def None(): pass", "__debug__ = 1"):
    try:
        compile(stmt, "<src>", "exec")
        print("keyword_target:", repr(stmt), "no_raise")
    except SyntaxError:
        pass
print("keyword_target: SyntaxError")


# A bad line-continuation indentation is an IndentationError.
try:
    exec("\\\nif 1:\n    y = 1\n  \\\n  z = 1\n")
    print("badindent: no_raise")
except IndentationError as e:
    print("badindent:", type(e).__name__)


# Malformed expression in eval is a SyntaxError.
for src in ("0x", ".. .", "3-4e/21"):
    try:
        eval(src)
        print("eval_bad:", repr(src), "no_raise")
    except SyntaxError:
        pass
print("eval_bad: SyntaxError")
