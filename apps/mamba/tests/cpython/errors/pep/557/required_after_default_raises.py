# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "required_after_default_raises"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba @dataclass decorator does not validate field ordering (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "errors.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a required field after a defaulted field raises TypeError at class creation"""
from dataclasses import dataclass

_raised = False
try:
    @dataclass
    class Order:
        a: int = 1
        b: int  # type: ignore[misc]
except TypeError:
    _raised = True
assert _raised, "required_after_default_raises: expected TypeError"
print("required_after_default_raises OK")
