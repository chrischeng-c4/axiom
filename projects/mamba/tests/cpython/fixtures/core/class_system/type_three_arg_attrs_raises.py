# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `type(name, bases, ns)` 3-arg constructor doesn't
# install attributes from the namespace dict on mamba; CPython
# does. See project_mamba_class_machinery_silent_divergences (#7).
"""type(name, bases, ns) class attr install (CPython 3.12 oracle)."""

NewCls = type("NewCls", (), {"x": 10, "label": "alpha"})

# CPython: 'NewCls'; mamba: 'NewCls' (name still works).
print("name:", NewCls.__name__)

# CPython: 10; mamba: None.
print("x:", NewCls.x)
# CPython: 'alpha'; mamba: None.
print("label:", NewCls.label)

# CPython: True; mamba: False (attrs absent).
print("hasattr_x:", hasattr(NewCls, "x"))

# Even on an instance: CPython: 10; mamba: None.
inst = NewCls()
print("inst_x:", inst.x)
