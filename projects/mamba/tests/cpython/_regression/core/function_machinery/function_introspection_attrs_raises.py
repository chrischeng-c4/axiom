# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `fn.__code__`, `fn.__defaults__`, `fn.__closure__`,
# `fn.__globals__` all return None on mamba; CPython exposes the
# real introspection objects. Accessing attributes on the None
# values raises AttributeError on mamba; CPython yields data. See
# project_mamba_function_machinery_silent_divergences (#8).
"""Function introspection attributes (CPython 3.12 oracle)."""


def make() -> object:
    cell = 100

    def fn(a: int = 7, b: int = 8) -> int:
        return cell + a + b

    return fn


fn = make()

# CPython: 'tuple'; mamba: 'NoneType' (defaults is None).
print("defaults_type:", type(fn.__defaults__).__name__)
# CPython: (7, 8); mamba: AttributeError on indexing None.
try:
    print("defaults:", tuple(fn.__defaults__))
except (TypeError, AttributeError) as e:
    print(f"defaults: {type(e).__name__}: {str(e)[:40]}")

# CPython: 'code'; mamba: 'NoneType'.
print("code_type:", type(fn.__code__).__name__)
try:
    print("co_name:", fn.__code__.co_name)
except AttributeError as e:
    print(f"co_name: AttributeError: {str(e)[:40]}")

# CPython: tuple of cell objects; mamba: NoneType.
print("closure_type:", type(fn.__closure__).__name__)
