# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "frozen_mutation_rejected"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not honor frozen synthesized __setattr__ (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "surface.py"
# status = "filled"
# ///
"""dataclasses.dataclass: frozen=True read access works but assigning a field on a frozen instance is rejected (raises)"""
from dataclasses import dataclass


@dataclass(frozen=True)
class Const:
    value: int


k = Const(42)
assert k.value == 42, f"const = {k.value!r}"

_raised = False
try:
    k.value = 99  # type: ignore[misc]
except Exception:
    _raised = True
assert _raised, "frozen dataclass should reject mutation"
print("frozen_mutation_rejected OK")
