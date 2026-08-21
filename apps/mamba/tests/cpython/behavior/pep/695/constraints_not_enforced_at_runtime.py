# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "constraints_not_enforced_at_runtime"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: type-param constraints are unenforced: def pick[T: (int, str)](x) returns 1, 'z', and even 3.5 (a float matching neither constraint)"""


# Constraints are likewise unenforced at runtime.
def pick[T: (int, str)](x: T) -> T:
    return x


assert pick(1) == 1
assert pick("z") == "z"
assert pick(3.5) == 3.5  # float matches neither constraint, but still runs

print("constraints_not_enforced_at_runtime OK")
