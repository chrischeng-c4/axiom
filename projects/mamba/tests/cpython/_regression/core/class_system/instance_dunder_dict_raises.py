# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `instance.__dict__` is None on mamba; CPython returns
# the attribute mapping. See
# project_mamba_class_machinery_silent_divergences (#4).
"""Instance __dict__ attribute (CPython 3.12 oracle)."""


class D:
    def __init__(self):
        self.x = 1
        self.y = "two"


d = D()
print("dict_type:", type(d.__dict__).__name__)
# CPython: True; mamba: False.
print("is_none:", d.__dict__ is None)

# CPython: ok; mamba: AttributeError.
try:
    keys = sorted(d.__dict__.keys())
    print("keys:", keys)
except AttributeError as e:
    print(f"keys: AttributeError: {str(e)[:40]}")
