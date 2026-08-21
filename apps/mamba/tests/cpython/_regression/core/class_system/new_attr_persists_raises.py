# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: attributes set inside `__new__` are not visible on the
# instance returned to `__init__`/caller on mamba; CPython preserves
# them. See project_mamba_class_machinery_silent_divergences (#2).
"""Attributes set in __new__ persist (CPython 3.12 oracle)."""


class N:
    def __new__(cls):
        inst = super().__new__(cls)
        inst.created_by_new = True
        return inst

    def __init__(self):
        self.created_by_init = True


n = N()
# CPython: True; mamba: None (silently dropped).
print("by_new:", getattr(n, "created_by_new", None))
# CPython: True; mamba: True (init-set attrs survive).
print("by_init:", getattr(n, "created_by_init", None))

# Defensive: hasattr check.
print("has_new_attr:", hasattr(n, "created_by_new"))
