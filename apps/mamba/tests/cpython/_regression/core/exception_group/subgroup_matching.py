# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/exception_group: subgroup()/split() matching by type and arg validation (3.12)."""


def make_eg():
    return ExceptionGroup(
        "eg", [ValueError(1), TypeError(2), KeyError(3), ValueError(4)]
    )


eg = make_eg()

# subgroup(type) keeps only matching leaves; identity passthrough when all match.
vals = eg.subgroup(ValueError)
assert [type(e).__name__ for e in vals.exceptions] == ["ValueError", "ValueError"]
assert eg is eg.subgroup(Exception)
assert eg is eg.subgroup(BaseException)
assert eg is eg.subgroup(BaseExceptionGroup)
assert eg is eg.subgroup(ExceptionGroup)

# subgroup returns None when nothing matches the type.
assert eg.subgroup(OSError) is None

# A tuple of types matches any of them.
some = eg.subgroup((ValueError, KeyError))
assert [type(e).__name__ for e in some.exceptions] == [
    "ValueError",
    "KeyError",
    "ValueError",
]

# split(type) returns (matching, rest); rest holds the complement.
match, rest = eg.split(KeyError)
assert [type(e).__name__ for e in match.exceptions] == ["KeyError"]
assert [type(e).__name__ for e in rest.exceptions] == [
    "ValueError",
    "TypeError",
    "ValueError",
]

# Bad argument types raise TypeError for both subgroup and split. Valid args
# are a type, a tuple of types, or a callable -- anything else is rejected.
bad_args = ["bad arg", OSError("instance not type"), [OSError, TypeError], (OSError, 42)]
for arg in bad_args:
    for op in (lambda a=arg: eg.subgroup(a), lambda a=arg: eg.split(a)):
        try:
            op()
            raise AssertionError(f"expected TypeError for {arg!r}")
        except TypeError:
            pass

print("subgroup_matching OK")
