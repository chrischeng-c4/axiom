# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `a += other` with `__iadd__` returning self assigns
# None to `a` on mamba; CPython preserves the return value. See
# project_mamba_class_machinery_silent_divergences (#10).
"""__iadd__ return-value preservation (CPython 3.12 oracle)."""


class Bag:
    def __init__(self):
        self.items: list = []

    def __iadd__(self, other):
        self.items.append(other)
        return self


a = Bag()
a += 1
# CPython: 'Bag'; mamba: 'NoneType'.
print("type_after_iadd:", type(a).__name__)

# CPython: False; mamba: True.
print("is_none:", a is None)

# Reset and use the workaround: explicit add.
b = Bag()
b.items.append(99)
# CPython: 'Bag'; mamba: 'Bag' (workaround works).
print("after_workaround_type:", type(b).__name__)
