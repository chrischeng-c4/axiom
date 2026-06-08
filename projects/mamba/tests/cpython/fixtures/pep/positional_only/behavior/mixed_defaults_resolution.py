# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "mixed_defaults_resolution"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a mixed pos-only/regular/kw-only signature with defaults resolves each override correctly: def f(po, /, reg=10, *, ko=100) yields 111 / 103 / 16 / 6 across the override combinations"""

# Rule: defaults across pos-only / regular / kw-only resolve independently as
# each override combination is applied.
def _complex(po: int, /, reg: int = 10, *, ko: int = 100) -> int:
    return po + reg + ko

assert _complex(1) == 111, _complex(1)
assert _complex(1, 2) == 103, _complex(1, 2)
assert _complex(1, ko=5) == 16, _complex(1, ko=5)
assert _complex(1, 2, ko=3) == 6, _complex(1, 2, ko=3)

print("mixed_defaults_resolution OK")
