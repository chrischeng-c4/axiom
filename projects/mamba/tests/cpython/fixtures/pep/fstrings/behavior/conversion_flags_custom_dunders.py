# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "conversion_flags_custom_dunders"
# subject = "fstring.conversion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.conversion: !r calls __repr__ and !s calls __str__ on a custom class: a class with __repr__ 'Custom(repr)' and __str__ 'Custom(str)' gives f'{c!r}' 'Custom(repr)' and f'{c!s}' 'Custom(str)'"""
# conversion flags dispatch the matching dunder on a custom type

class Custom:
    def __repr__(self) -> str:
        return "Custom(repr)"
    def __str__(self) -> str:
        return "Custom(str)"

c = Custom()
assert f"{c!r}" == "Custom(repr)", f"!r = {f'{c!r}'!r}"
assert f"{c!s}" == "Custom(str)", f"!s = {f'{c!s}'!r}"

print("conversion_flags_custom_dunders OK")
