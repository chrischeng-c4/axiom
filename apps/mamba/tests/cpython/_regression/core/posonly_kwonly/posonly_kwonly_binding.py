# Positional-only and keyword-only argument binding — #2781.
#
# Covers Python's `/` and `*` parameter markers introduced in
# PEP 570 (positional-only) and PEP 3102 (keyword-only).
#
#   def f(a, b, /, c, d, *, e, f):
#       ...
#
#   - `a`, `b`         positional-only (cannot be passed by keyword).
#   - `c`, `d`         positional-or-keyword (both work).
#   - `e`, `f`         keyword-only (cannot be passed positionally).
#
# Clauses:
#   1. Positional-only happy path — passing values positionally
#      succeeds and binds them correctly.
#   2. Positional-only TypeError — attempting to pass a posonly
#      argument by keyword raises TypeError.
#   3. Keyword-only happy path — passing kwonly args by keyword.
#   4. Keyword-only TypeError — attempting to pass a kwonly arg
#      positionally raises TypeError.
#   5. Positional-or-keyword middle section accepts both styles.
#   6. Positional-only kwarg name is free for **kwargs absorption.
#      Because the name is unreachable via keyword on the function
#      itself, an explicit kwarg of that name lands in **kwargs.
#
# Every print line tagged `[posonly-kwonly]` so failure output
# names the binding semantics.


def f(a, b, /, c, d, *, e, f):
    return (a, b, c, d, e, f)


# Clause 1: positional-only happy path.
print(
    "[posonly-kwonly] clause-1 all-positional-then-kwonly:",
    f(1, 2, 3, 4, e=5, f=6),
)
print(
    "[posonly-kwonly] clause-1 mixed:",
    f(1, 2, 3, d=4, e=5, f=6),
)


# Clause 2: passing `a` (positional-only) by keyword must fail.
try:
    f(a=1, b=2, c=3, d=4, e=5, f=6)  # pyright: ignore[reportCallIssue]
    print("[posonly-kwonly] clause-2 typeerror: <unexpected-no-error>")
except TypeError as exc:
    print("[posonly-kwonly] clause-2 typeerror:", type(exc).__name__)


# Clause 3: keyword-only happy path is the canonical call shape.
print(
    "[posonly-kwonly] clause-3 kwonly-by-keyword:",
    f(1, 2, 3, 4, e=50, f=60),
)


# Clause 4: passing `e` (keyword-only) positionally must fail.
try:
    f(1, 2, 3, 4, 5, 6)  # pyright: ignore[reportCallIssue]
    print("[posonly-kwonly] clause-4 typeerror: <unexpected-no-error>")
except TypeError as exc:
    print("[posonly-kwonly] clause-4 typeerror:", type(exc).__name__)


# Clause 5: positional-or-keyword middle section accepts both.
print(
    "[posonly-kwonly] clause-5 middle-positional:",
    f(1, 2, 3, 4, e=5, f=6),
)
print(
    "[posonly-kwonly] clause-5 middle-keyword:",
    f(1, 2, c=3, d=4, e=5, f=6),
)


# Clause 6: positional-only name is reusable as a **kwargs key. This
# function's `name` is positional-only; the caller can still pass a
# `name=...` keyword and it lands in **extras because the function's
# own `name` is unreachable via keyword.
def g(name, /, **extras):
    return name, sorted(extras.items())


print(
    "[posonly-kwonly] clause-6 posonly-name-free-for-kwargs:",
    g("alice", name="bob", role="admin"),
)
