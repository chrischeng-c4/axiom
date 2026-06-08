# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "errors"
# case = "mutable_list_default_raises"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba @dataclass decorator does not validate mutable defaults (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "errors.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a bare mutable list default (items: list = []) raises ValueError at class creation"""
from dataclasses import dataclass

_raised = False
try:
    @dataclass
    class Bad:
        items: list = []  # type: ignore[assignment]
except ValueError:
    _raised = True
assert _raised, "mutable_list_default_raises: expected ValueError"
print("mutable_list_default_raises OK")
