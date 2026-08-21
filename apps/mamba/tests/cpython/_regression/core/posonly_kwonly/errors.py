# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/posonly_kwonly: positional-only / keyword-only error paths (CPython 3.12 oracle)."""


def f(a, b, *, c):
    return (a, b, c)


# Missing required kw-only raises TypeError.
try:
    f(1, 2)  # type: ignore[call-arg]
    print("missing_kw: no_raise")
except TypeError as e:
    print("missing_kw:", type(e).__name__, str(e)[:60])


# Too many positional args raises TypeError.
def g(a, b):
    return a + b


try:
    g(1, 2, 3)  # type: ignore[call-arg]
    print("too_many: no_raise")
except TypeError as e:
    print("too_many:", type(e).__name__, str(e)[:60])


# Duplicate keyword arg raises TypeError.
try:
    g(1, a=2)  # type: ignore[call-arg]
    print("dup_kw: no_raise")
except TypeError as e:
    print("dup_kw:", type(e).__name__, str(e)[:60])


# Passing a positional-only param by keyword is rejected with a
# distinctive message naming the offending parameter(s).
def posonly(a, b, /):
    return (a, b)


try:
    posonly(a=1, b=2)  # type: ignore[call-arg]
    print("posonly_as_kw: no_raise")
except TypeError as e:
    msg = str(e)
    assert "positional-only arguments passed as keyword arguments" in msg, msg
    assert "'a, b'" in msg, msg
    print("posonly_as_kw: TypeError ok")


# Mixed positional-only + keyword-only: omitting required kw-only args
# names them in the error.
def mixed(a, b, /, c, *, d, e):
    return (a, b, c, d, e)


try:
    mixed(1, 2, 3, e=2)  # type: ignore[call-arg]
    print("missing_one_kwonly: no_raise")
except TypeError as e:
    msg = str(e)
    assert "missing 1 required keyword-only argument: 'd'" in msg, msg
    print("missing_one_kwonly: TypeError ok")

try:
    mixed(1, 2, 3)  # type: ignore[call-arg]
    print("missing_two_kwonly: no_raise")
except TypeError as e:
    msg = str(e)
    assert "missing 2 required keyword-only arguments: 'd' and 'e'" in msg, msg
    print("missing_two_kwonly: TypeError ok")


# Over-supplying positionals when kw-only args are also present yields a
# message that counts positional and keyword-only segments separately.
try:
    mixed(1, 2, 3, 4, 5, 6, d=7, e=8)  # type: ignore[call-arg]
    print("over_positional: no_raise")
except TypeError as e:
    msg = str(e)
    assert "takes 3 positional arguments but 6 positional arguments" in msg, msg
    assert "keyword-only arguments) were given" in msg, msg
    print("over_positional: TypeError ok")


# An unrecognized keyword is reported as unexpected.
try:
    mixed(1, 2, 3, d=1, e=4, zzz=56)  # type: ignore[call-arg]
    print("unexpected_kw: no_raise")
except TypeError as e:
    msg = str(e)
    assert "got an unexpected keyword argument 'zzz'" in msg, msg
    print("unexpected_kw: TypeError ok")
