# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "non_callable_default_factory_raises"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba @dataclass decorator does not validate default_factory callability (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "errors.py"
# status = "filled"
# ///
"""dataclasses.field: field(default_factory=42) with a non-callable factory raises TypeError at __init__"""
from dataclasses import dataclass, field

_raised = False
try:
    @dataclass
    class BadFactory:
        items: list = field(default_factory=42)  # type: ignore[arg-type]
    BadFactory()
except TypeError:
    _raised = True
assert _raised, "non_callable_default_factory_raises: expected TypeError"
print("non_callable_default_factory_raises OK")
