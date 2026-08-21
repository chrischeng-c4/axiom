# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `instance.__class__` returns None on mamba; CPython
# points back to the type. `type(instance)` still works. See
# project_mamba_class_machinery_silent_divergences (#8).
"""Instance __class__ attribute (CPython 3.12 oracle)."""


class K:
    pass


k = K()

# CPython: 'type'; mamba: 'NoneType'.
print("dunder_type:", type(k.__class__).__name__)
# CPython: False; mamba: True.
print("dunder_is_none:", k.__class__ is None)

# CPython: 'K'; mamba: AttributeError.
try:
    print("dunder_name:", k.__class__.__name__)
except AttributeError as e:
    print(f"dunder_name: AttributeError: {str(e)[:40]}")

# type(k) still works on both.
print("type_name:", type(k).__name__)
print("class_is_type:", k.__class__ is type(k))
