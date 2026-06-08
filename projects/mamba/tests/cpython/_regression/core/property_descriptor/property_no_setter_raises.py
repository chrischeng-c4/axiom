# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: assigning to a property without a setter silently
# no-ops on mamba; CPython raises AttributeError "property 'v' of
# 'PR' object has no setter". See
# project_mamba_class_machinery_silent_divergences (#9).
"""Property without setter rejects assignment (CPython 3.12 oracle)."""


class PR:
    def __init__(self):
        self._v = 1

    @property
    def v(self):
        return self._v


pr = PR()
# CPython: 1; mamba: 1.
print("get_initial:", pr.v)

# CPython: AttributeError "has no setter"; mamba: silent.
try:
    pr.v = 99
    print(f"no_raise; new_v: {pr.v}")
except AttributeError as e:
    print(f"AttributeError: {str(e)[:60]}")

# Defensive: did the backing store change?
print("backing:", pr._v)
