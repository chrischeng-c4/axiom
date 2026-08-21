# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: decorator-with-*args/**kwargs wrap drops the inner
# return value on mamba; CPython preserves it. See
# project_mamba_function_machinery_silent_divergences (#2).
"""PEP 318 decorator with *args/**kwargs wrap (CPython 3.12 oracle)."""


def tag(fn):
    def wrap(*a, **kw):
        return ("wrapped", fn(*a, **kw))
    return wrap


@tag
def inner(x: int, y: int) -> int:
    return x + y


# CPython: ("wrapped", 7).
# Mamba: ("wrapped", 0) — inner's return is dropped.
result = inner(3, 4)
print("result:", result)

# CPython: True; mamba: False (since inner returned 0).
print("preserved:", result == ("wrapped", 7))

# Defensive: catch any TypeError mamba may throw on the call itself.
try:
    print("type:", type(result).__name__)
except TypeError as e:
    print(f"type: TypeError: {str(e)[:40]}")
