# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `cls.__subclasses__()` raises AttributeError on mamba
# (name lookup goes to a string internally); CPython returns a list
# of direct subclasses. See
# project_mamba_class_machinery_silent_divergences (#6).
"""Class __subclasses__() method (CPython 3.12 oracle)."""


class Base: pass
class A(Base): pass
class B(Base): pass


# CPython: ['A', 'B']; mamba: AttributeError.
try:
    subs = sorted(c.__name__ for c in Base.__subclasses__())
    print("subs:", subs)
except AttributeError as e:
    print(f"AttributeError: {str(e)[:60]}")

# Empty for a leaf class.
try:
    leaf_subs = list(A.__subclasses__())
    print("leaf_subs:", leaf_subs)
except AttributeError as e:
    print(f"leaf_subs: AttributeError: {str(e)[:40]}")
