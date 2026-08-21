# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/exception_group: read-only fields, note copying on split, derive guard (3.12)."""


# .exceptions is a tuple; both message and exceptions are read-only.
eg = ExceptionGroup("eg", [TypeError(1), OSError(2)])
assert type(eg.exceptions) is tuple
assert eg.message == "eg"
for field in ("message", "exceptions"):
    try:
        setattr(eg, field, "x")
        raise AssertionError(f"expected AttributeError on {field}")
    except AttributeError:
        pass


# split() copies __notes__ into independent lists on match and rest.
eg = ExceptionGroup("eg", [ValueError(1), TypeError(2)])
eg.add_note("note1")
eg.add_note("note2")
orig = list(eg.__notes__)
match, rest = eg.split(TypeError)
assert eg.__notes__ == orig
assert match.__notes__ == orig
assert rest.__notes__ == orig
assert eg.__notes__ is not match.__notes__
assert eg.__notes__ is not rest.__notes__
assert match.__notes__ is not rest.__notes__
# Mutating one does not bleed into the others.
eg.add_note("eg")
match.add_note("match")
rest.add_note("rest")
assert eg.__notes__ == orig + ["eg"]
assert match.__notes__ == orig + ["match"]
assert rest.__notes__ == orig + ["rest"]


# A non-sequence __notes__ is not propagated by split().
weird = ExceptionGroup("eg", [ValueError(1), TypeError(2)])
weird.__notes__ = 123
m2, r2 = weird.split(TypeError)
assert not hasattr(m2, "__notes__")
assert not hasattr(r2, "__notes__")


# split()/subgroup() require derive() to return a BaseExceptionGroup.
class BadDerive(ExceptionGroup):
    def derive(self, excs):
        return 42


bad = BadDerive("eg", [TypeError(1), ValueError(2)])
for op in (lambda: bad.split(TypeError), lambda: bad.subgroup(TypeError)):
    try:
        op()
        raise AssertionError("expected TypeError")
    except TypeError as e:
        assert "derive must return an instance of BaseExceptionGroup" in str(e), e

print("fields_and_notes OK")
