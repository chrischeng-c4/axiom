# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "default_factory_runs_only_when_omitted"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = "mamba does not synthesize default_factory in __init__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "field_defaults.py"
# status = "filled"
# ///
"""dataclasses.field: the factory runs only when no explicit value is passed, and runs again on each defaulted construction"""
from dataclasses import dataclass, field

calls = []


def make():
    calls.append(1)
    return len(calls)


@dataclass
class Counted:
    n: int = field(default_factory=make)


assert Counted().n == 1
assert len(calls) == 1
assert Counted(99).n == 99  # explicit value -> factory skipped
assert len(calls) == 1
assert Counted().n == 2  # factory runs again
assert len(calls) == 2
print("default_factory_runs_only_when_omitted OK")
