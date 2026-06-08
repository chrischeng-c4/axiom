# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `cls.__bases__` is None on mamba; CPython returns the
# tuple of direct bases. `isinstance` / `issubclass` still work
# because bases are tracked internally. See
# project_mamba_class_machinery_silent_divergences (#5).
"""Class __bases__ tuple (CPython 3.12 oracle)."""


class B: pass
class C(B): pass


# CPython: 'tuple'; mamba: 'NoneType'.
print("type:", type(C.__bases__).__name__)
# CPython: False; mamba: True.
print("is_none:", C.__bases__ is None)
# CPython: ['B']; mamba: AttributeError.
try:
    names = [b.__name__ for b in C.__bases__]
    print("names:", names)
except (TypeError, AttributeError) as e:
    print(f"names: {type(e).__name__}: {str(e)[:40]}")

# isinstance / issubclass still work.
print("issub:", issubclass(C, B))
print("isinst:", isinstance(C(), B))
