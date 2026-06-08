# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `str(pathlib.Path('/tmp/foo/bar.txt'))`
# (the documented "str of a Path renders the underlying path text" —
# mamba returns '<PosixPath instance>'), `Path(...).name` (the
# documented ".name returns the final path segment" — mamba returns
# None), `Path(...).stem` (the documented ".stem returns the final
# segment with the suffix stripped" — mamba returns None),
# `Path(...).suffix` (the documented ".suffix returns the trailing
# '.ext' part of the final segment" — mamba returns None),
# `str(Path(...).parent)` (the documented ".parent returns the parent
# Path whose str is the directory path" — mamba returns 'None'),
# `Path(...).parts` (the documented ".parts returns a tuple of path
# segments" — mamba returns None), `str(Path('/tmp') / 'foo' / 'bar')`
# (the documented "'/' operator joins path components" — mamba
# returns 'None'), `Path(...).with_suffix('.py')` (the documented
# "returns a new Path with the suffix replaced" — mamba raises
# AttributeError), `Path(...).is_absolute()` (the documented
# "predicate returning whether the path is absolute" — mamba raises
# AttributeError), and `str(uuid.uuid4()).count('-')` (the documented
# "str of a UUID renders the canonical 8-4-4-4-12 hyphen-separated
# form" — mamba returns 0 because str(uuid) lacks hyphens).
# Ten-pack pinned to atomic 257.
#
# Behavioral edges that CONFORM on mamba (random — hasattr random/
# randint/choice/shuffle/sample/seed/randrange/uniform/gauss/Random/
# getrandbits/choices + seeded random ∈ [0,1), randint bounded,
# choice in list, shuffle preserves, sample length, randrange upper
# bound. secrets — hasattr token_bytes/token_hex/token_urlsafe/
# choice/randbelow/compare_digest + token_bytes(16) length, token_hex
# (8) length 16, randbelow upper bound, compare_digest True/False.
# uuid — hasattr uuid1/uuid3/uuid4/uuid5/UUID/NAMESPACE_DNS +
# uuid4().hex length 32 and hex-charset only. pathlib — hasattr
# Path/PurePath/PurePosixPath/PureWindowsPath/PosixPath) are covered
# in the matching pass fixture
# `test_random_secrets_uuid_pathlib_value_ops`.
import pathlib
import uuid
from typing import Any


_ledger: list[int] = []

# 1) str(Path(...)) renders the path text
#    (mamba: returns '<PosixPath instance>')
assert str(pathlib.Path("/tmp/foo/bar.txt")) == "/tmp/foo/bar.txt"; _ledger.append(1)

# 2) Path.name — final segment
#    (mamba: returns None)
assert pathlib.Path("/tmp/foo/bar.txt").name == "bar.txt"; _ledger.append(1)

# 3) Path.stem — final segment without suffix
#    (mamba: returns None)
assert pathlib.Path("/tmp/foo/bar.txt").stem == "bar"; _ledger.append(1)

# 4) Path.suffix — trailing '.ext'
#    (mamba: returns None)
assert pathlib.Path("/tmp/foo/bar.txt").suffix == ".txt"; _ledger.append(1)

# 5) str(Path.parent) — directory of the path
#    (mamba: returns 'None')
assert str(pathlib.Path("/tmp/foo/bar.txt").parent) == "/tmp/foo"; _ledger.append(1)

# 6) Path.parts — tuple of segments
#    (mamba: returns None)
assert pathlib.Path("/tmp/foo/bar.txt").parts == ("/", "tmp", "foo", "bar.txt"); _ledger.append(1)

# 7) str(Path / 'sub' / 'file') — '/' join
#    (mamba: returns 'None')
assert str(pathlib.Path("/tmp") / "foo" / "bar.txt") == "/tmp/foo/bar.txt"; _ledger.append(1)

# 8) Path.with_suffix('.py') replaces the suffix
#    (mamba: AttributeError)
def _with_suffix() -> Any:
    try:
        return str(pathlib.Path("/tmp/foo/bar.txt").with_suffix(".py"))
    except AttributeError:
        return None
assert _with_suffix() == "/tmp/foo/bar.py"; _ledger.append(1)

# 9) Path.is_absolute() returns True for absolute paths
#    (mamba: AttributeError)
def _is_absolute() -> Any:
    try:
        return pathlib.Path("/tmp/foo/bar.txt").is_absolute()
    except AttributeError:
        return None
assert _is_absolute() == True; _ledger.append(1)

# 10) str(uuid.uuid4()) renders 8-4-4-4-12 hyphen-separated form
#     (mamba: returns hex without hyphens — count == 0)
assert str(uuid.uuid4()).count("-") == 4; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pathlib_uuid_silent {sum(_ledger)} asserts")
