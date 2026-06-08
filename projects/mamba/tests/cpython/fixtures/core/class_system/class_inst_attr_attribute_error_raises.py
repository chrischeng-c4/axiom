# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: accessing an instance-only attribute via the class name
# silently returns None on mamba; CPython raises AttributeError. See
# project_mamba_class_machinery_silent_divergences (#3).
"""Class-level access to instance attr raises AttributeError (CPython 3.12 oracle)."""


class O:
    def __init__(self):
        self.inst_attr = 7


# CPython: AttributeError "type object 'O' has no attribute 'inst_attr'".
# Mamba: silently returns None.
try:
    v = O.inst_attr
    print(f"no_raise: {v!r}")
except AttributeError as e:
    print(f"AttributeError: {str(e)[:60]}")

# Instance access works on both runtimes.
print("via_instance:", O().inst_attr)
