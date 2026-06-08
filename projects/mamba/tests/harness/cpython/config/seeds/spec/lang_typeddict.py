# lang_typeddict.py — #3346 axis-1 lang TypedDict AssertionPass seed.
#
# Mamba-authored seed exercising the `typing.TypedDict` surface called
# out in the issue:
#   class Movie(TypedDict): title: str; year: int
#   Movie(title="x", year=1) returns plain dict
#   Required[str] / NotRequired[int] annotations
#   total=False makes all keys NotRequired by default
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3493 (TypedDict subclass produces wrapper instance, not dict) blocks
# AssertionPass today. Once #3493 lands and this seed flips to
# AssertionPass on mamba, drift detection prompts a
# `git mv spec/lang_typeddict.py pass/lang_typeddict.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + import.
#   2. Class-based TypedDict: `class Movie(TypedDict): title: str; year: int`.
#   3. Movie(title=..., year=...) returns a plain dict (NOT a wrapper).
#   4. Movie inherits dict identity (isinstance(m, dict) True).
#   5. __required_keys__ / __optional_keys__ / __total__ class attrs.
#   6. total=False — all keys land in __optional_keys__.
#   7. Required[T] / NotRequired[T] in a total=True TypedDict — overrides
#      defaults per-field.
#   8. Functional-form TypedDict (legacy): `Foo = TypedDict("Foo", {...})`.
#   9. __annotations__ exposes the declared fields.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_typeddict N asserts` to stdout.

from typing import TypedDict, Required, NotRequired

_ledger: list[int] = []


# 2. Class-based TypedDict declaration.
class Movie(TypedDict):
    title: str
    year: int


# 3. Instantiation returns a plain dict.
_m = Movie(title="Inception", year=2010)
assert isinstance(_m, dict), "Movie(...) returns a dict instance (not wrapper)"
_ledger.append(1)
# 4. dict semantics: subscript + keyword construction wired through.
assert _m["title"] == "Inception", "Movie['title'] dict access"
_ledger.append(1)
assert _m["year"] - 2010 == 0, "Movie['year'] dict access (boxed-dodge)"
_ledger.append(1)
assert len(_m) - 2 == 0, "Movie has 2 keys (boxed-dodge)"
_ledger.append(1)

# 5. TypedDict class-level introspection.
assert hasattr(Movie, "__required_keys__"), "Movie.__required_keys__ exposed"
_ledger.append(1)
assert hasattr(Movie, "__optional_keys__"), "Movie.__optional_keys__ exposed"
_ledger.append(1)
assert hasattr(Movie, "__total__"), "Movie.__total__ exposed"
_ledger.append(1)
assert Movie.__total__ == True, "Movie default total=True"
_ledger.append(1)
# All declared fields required by default (total=True).
assert "title" in Movie.__required_keys__, "'title' is required by default"
_ledger.append(1)
assert "year" in Movie.__required_keys__, "'year' is required by default"
_ledger.append(1)
assert len(Movie.__optional_keys__) == 0, "no optional keys under total=True"
_ledger.append(1)


# 6. total=False — all keys are optional.
class PartialMovie(TypedDict, total=False):
    title: str
    year: int


assert PartialMovie.__total__ == False, "PartialMovie total=False"
_ledger.append(1)
assert "title" in PartialMovie.__optional_keys__, (
    "'title' optional under total=False"
)
_ledger.append(1)
assert "year" in PartialMovie.__optional_keys__, (
    "'year' optional under total=False"
)
_ledger.append(1)
assert len(PartialMovie.__required_keys__) == 0, (
    "no required keys under total=False"
)
_ledger.append(1)
# Empty construction allowed because all keys are optional.
_pm: PartialMovie = PartialMovie()
assert isinstance(_pm, dict), "PartialMovie() is a dict"
_ledger.append(1)
assert len(_pm) == 0, "PartialMovie() empty when no kwargs supplied"
_ledger.append(1)


# 7. Required[T] / NotRequired[T] inside total=True override per-field.
class Book(TypedDict):
    title: Required[str]
    isbn: NotRequired[str]


assert Book.__total__ == True, "Book default total=True"
_ledger.append(1)
assert "title" in Book.__required_keys__, "title still required when Required[str]"
_ledger.append(1)
assert "isbn" in Book.__optional_keys__, "isbn optional via NotRequired[str]"
_ledger.append(1)


# 7b. NotRequired inside total=False — alias works.
class BookOpt(TypedDict, total=False):
    title: Required[str]
    isbn: str


assert "title" in BookOpt.__required_keys__, (
    "Required[str] overrides total=False default"
)
_ledger.append(1)
assert "isbn" in BookOpt.__optional_keys__, (
    "plain str field stays optional under total=False"
)
_ledger.append(1)


# 8. Functional-form TypedDict (legacy syntax — call form).
Point = TypedDict("Point", {"x": int, "y": int})
_p = Point(x=3, y=4)
assert isinstance(_p, dict), "functional-form TypedDict instance is a dict"
_ledger.append(1)
assert _p["x"] - 3 == 0, "functional-form Point['x'] (boxed-dodge)"
_ledger.append(1)
assert _p["y"] - 4 == 0, "functional-form Point['y'] (boxed-dodge)"
_ledger.append(1)

# 9. __annotations__ on TypedDict classes.
assert hasattr(Movie, "__annotations__"), "Movie.__annotations__ exposed"
_ledger.append(1)
assert "title" in Movie.__annotations__, "Movie.__annotations__ contains 'title'"
_ledger.append(1)
assert "year" in Movie.__annotations__, "Movie.__annotations__ contains 'year'"
_ledger.append(1)
# Annotated type for 'year' is int.
assert Movie.__annotations__["year"] is int, "Movie.__annotations__['year'] is int"
_ledger.append(1)
assert Movie.__annotations__["title"] is str, (
    "Movie.__annotations__['title'] is str"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_typeddict {len(_ledger)} asserts")
