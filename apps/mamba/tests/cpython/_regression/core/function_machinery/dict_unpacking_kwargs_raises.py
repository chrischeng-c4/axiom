# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `**dict` call-site unpacking passes the dict as the
# first positional arg on mamba where CPython expands to keyword args.
# See project_mamba_function_machinery_silent_divergences (#1).
"""PEP/protocol: **dict call-site unpacking (CPython 3.12 oracle)."""


def take(a: int, b: int, c: int) -> int:
    return a * 100 + b * 10 + c


# CPython expands {'a':1,'b':2,'c':3} into a=1, b=2, c=3 -> 123.
# Mamba passes the dict as the first positional arg -> TypeError or 100*dict.
try:
    result = take(**{"a": 1, "b": 2, "c": 3})
    print(f"expand: {result}")
except TypeError as e:
    print(f"expand: TypeError: {str(e)[:40]}")

# A second call with conflicting positional + **kwargs should raise.
try:
    take(1, **{"a": 1, "b": 2, "c": 3})
    print("conflict: no_raise")
except TypeError as e:
    print(f"conflict: TypeError: {str(e)[:40]}")
