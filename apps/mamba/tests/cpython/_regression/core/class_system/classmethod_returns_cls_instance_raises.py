# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `@classmethod def make(cls): return cls()` returns None
# on mamba; CPython returns an instance of the class. See
# project_mamba_class_machinery_silent_divergences (#1).
"""classmethod calling cls() (CPython 3.12 oracle)."""


class Q:
    @classmethod
    def make(cls):
        return cls()


inst = Q.make()
# CPython: 'Q'; mamba: 'NoneType'.
print("type:", type(inst).__name__)
# CPython: True; mamba: False.
print("isinstance:", isinstance(inst, Q))

try:
    print("name:", type(inst).__name__)
except AttributeError as e:
    print(f"name: AttributeError: {str(e)[:40]}")
