# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/posonly_kwonly: language-area surface probes (CPython 3.12 oracle)."""

# Core language constructs always available.
import builtins
assert hasattr(builtins, "object")
assert hasattr(builtins, "type")
assert hasattr(builtins, "list")
assert hasattr(builtins, "dict")
assert hasattr(builtins, "tuple")
assert hasattr(builtins, "callable")

# Code-object introspection exposes the positional-only / keyword-only
# split via dedicated counters; a function with `/` and `*` markers
# reports each segment size separately.
def shape(a, b, c, /, d, e=1, *, f, g=2):
    return (a, b, c, d, e, f, g)

code = shape.__code__
assert code.co_argcount == 5, code.co_argcount          # a..e (posonly + pos-or-kw)
assert code.co_posonlyargcount == 3, code.co_posonlyargcount  # a, b, c
assert code.co_kwonlyargcount == 2, code.co_kwonlyargcount    # f, g

# Defaults are introspectable too: positional defaults live in
# __defaults__ (left-aligned to the tail of co_argcount), keyword-only
# defaults live in __kwdefaults__ keyed by name.
assert shape.__defaults__ == (1,), shape.__defaults__
assert shape.__kwdefaults__ == {"g": 2}, shape.__kwdefaults__

# Lambdas carry the same markers.
lam = lambda a, /, b, *, c: (a, b, c)
assert lam.__code__.co_posonlyargcount == 1
assert lam.__code__.co_kwonlyargcount == 1

print("surface OK")
