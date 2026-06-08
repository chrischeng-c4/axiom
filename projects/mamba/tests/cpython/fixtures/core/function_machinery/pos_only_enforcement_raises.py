# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: positional-only arguments are not enforced on mamba;
# CPython raises TypeError when a pos-only arg is passed by keyword.
# See project_mamba_function_machinery_silent_divergences (#4).
"""Positional-only argument enforcement (CPython 3.12 oracle)."""


def p(a: int, b: int, /) -> int:
    return a * 10 + b


# CPython: ok, returns 12.
print("pos_only_ok:", p(1, 2))

# CPython: TypeError "got some positional-only arguments passed as
# keyword arguments: 'a, b'". Mamba: silently accepts.
try:
    print("pos_as_kw:", p(a=1, b=2))
except TypeError as e:
    print(f"pos_as_kw: TypeError: {str(e)[:80]}")
