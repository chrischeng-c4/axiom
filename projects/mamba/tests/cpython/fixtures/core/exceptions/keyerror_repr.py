# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: KeyError formatting diverged from CPython three ways.
#
# 1. `print(KeyError("x"))` showed `x` — CPython shows `'x'` (KeyError
#    overrides __str__ to repr(args[0])).
# 2. `repr(KeyError("x"))` showed `x` — CPython shows `KeyError('x')`
#    (BaseException.__repr__ builds `ClassName(repr(args))`).
# 3. A missing dict lookup pre-quoted the key so the message stored on
#    the exception was already `'missing'`, which combined with (1) and
#    (2) gave `''missing''` after quoting was added.

# raise-and-catch path.
try:
    raise KeyError("hello")
except KeyError as e:
    print(e)
    print(str(e))
    print(repr(e))
    print(e.args)

# Dict-lookup-miss path — message must be the raw key, not a pre-quoted
# form, or str(e) will double-quote it.
try:
    d = {}
    _ = d["missing"]
except KeyError as e:
    print(e)
    print(str(e))
    print(repr(e))
    print(e.args)

# Non-KeyError exceptions must not accidentally adopt the quoting quirk.
try:
    raise ValueError("nope")
except ValueError as e:
    print(e)
    print(repr(e))