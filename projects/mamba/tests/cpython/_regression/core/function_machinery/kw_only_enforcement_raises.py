# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: keyword-only arguments are not enforced on mamba;
# CPython raises TypeError when a kw-only arg is passed positionally.
# See project_mamba_function_machinery_silent_divergences (#3).
"""Keyword-only argument enforcement (CPython 3.12 oracle)."""


def k(a: int, *, b: int) -> int:
    return a * 10 + b


# CPython: ok, returns 12.
print("kw_only_ok:", k(1, b=2))

# CPython: TypeError "takes 1 positional argument but 2 were given".
# Mamba: silently accepts as (1, 2).
try:
    print("kw_as_pos:", k(1, 2))
except TypeError as e:
    print(f"kw_as_pos: TypeError: {str(e)[:60]}")

# CPython: TypeError "missing 1 required keyword-only argument: 'b'".
try:
    print("missing_kw:", k(1))
except TypeError as e:
    print(f"missing_kw: TypeError: {str(e)[:60]}")
