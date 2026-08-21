# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "behavior"
# case = "annotation_only_does_not_bind"
# subject = "__annotations__"
# kind = "semantic"
# xfail = "module __annotations__ is an undefined name on mamba. See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: a bare annotation `y: int` (no value) does NOT bind the name at module scope (NameError on read) but DOES record 'y' in __annotations__"""

# A bare annotation records the name but never binds it.
y: int  # type-only annotation, no value
bound = True
try:
    y  # noqa: B018  -- read the unbound name
except NameError:
    bound = False
assert bound is False, "bare annotation must not bind the name"
assert "y" in __annotations__, __annotations__
print("annotation_only_does_not_bind OK")
