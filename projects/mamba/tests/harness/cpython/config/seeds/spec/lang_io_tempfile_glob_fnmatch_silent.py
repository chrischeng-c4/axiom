# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(io, 'IOBase')` (the
# documented "io exposes the IOBase abstract base class" — mamba
# returns False), `hasattr(io, 'open')` (the documented "io exposes
# the open() function as an alias for builtins.open" — mamba
# returns False), `hasattr(io, 'SEEK_SET')` (the documented "io
# exposes the SEEK_SET=0 seek-whence constant" — mamba returns
# False), `hasattr(io, 'UnsupportedOperation')` (the documented
# "io exposes the UnsupportedOperation exception" — mamba returns
# False), `hasattr(io, 'DEFAULT_BUFFER_SIZE')` (the documented
# "io exposes the DEFAULT_BUFFER_SIZE module constant" — mamba
# returns False), `io.SEEK_SET == 0` (the documented "SEEK_SET
# constant equals 0" — mamba returns None), `io.StringIO('hello').
# read()` (the documented "StringIO initialized with 'hello' reads
# back 'hello'" — mamba returns ''), `io.StringIO().write(...).
# getvalue()` (the documented "StringIO accumulates writes and
# getvalue returns them — returns 'hello'" — mamba returns ''),
# `io.BytesIO().write(...).getvalue()` (the documented "BytesIO
# accumulates byte writes — returns b'hello'" — mamba returns
# b''), and `type(io.StringIO()).__name__` (the documented
# "StringIO() returns a StringIO instance" — mamba returns 'dict').
# Ten-pack pinned to atomic 272.
#
# Behavioral edges that CONFORM on mamba (io — hasattr StringIO/
# BytesIO. tempfile — hasattr gettempdir/gettempprefix/mkdtemp/
# mkstemp/NamedTemporaryFile/TemporaryFile/TemporaryDirectory/
# SpooledTemporaryFile/tempdir + gettempdir non-empty str /
# gettempprefix str. glob — hasattr glob/iglob/escape/has_magic +
# escape '*'/'?'/'[]' + glob nonexistent. fnmatch — hasattr
# fnmatch/fnmatchcase/filter/translate + full value contracts) are
# covered in the matching pass fixture
# `test_io_tempfile_glob_fnmatch_value_ops`.
import io


_ledger: list[int] = []

# 1) hasattr(io, 'IOBase') — abstract base class
#    (mamba: returns False)
assert hasattr(io, "IOBase") == True; _ledger.append(1)

# 2) hasattr(io, 'open') — open() alias
#    (mamba: returns False)
assert hasattr(io, "open") == True; _ledger.append(1)

# 3) hasattr(io, 'SEEK_SET') — seek-whence constant
#    (mamba: returns False)
assert hasattr(io, "SEEK_SET") == True; _ledger.append(1)

# 4) hasattr(io, 'UnsupportedOperation') — exception class
#    (mamba: returns False)
assert hasattr(io, "UnsupportedOperation") == True; _ledger.append(1)

# 5) hasattr(io, 'DEFAULT_BUFFER_SIZE') — module constant
#    (mamba: returns False)
assert hasattr(io, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)

# 6) io.SEEK_SET == 0 — seek-whence constant value
#    (mamba: returns None)
assert io.SEEK_SET == 0; _ledger.append(1)

# 7) io.StringIO('hello').read() — read back constructor argument
#    (mamba: returns '')
assert io.StringIO("hello").read() == "hello"; _ledger.append(1)

# 8) StringIO().write/getvalue accumulation
#    (mamba: returns '')
_s = io.StringIO()
_s.write("hello")
assert _s.getvalue() == "hello"; _ledger.append(1)

# 9) BytesIO().write/getvalue accumulation
#    (mamba: returns b'')
_b = io.BytesIO()
_b.write(b"hello")
assert _b.getvalue() == b"hello"; _ledger.append(1)

# 10) type(io.StringIO()).__name__ == 'StringIO' — instance type
#     (mamba: returns 'dict')
assert type(io.StringIO()).__name__ == "StringIO"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_tempfile_glob_fnmatch_silent {sum(_ledger)} asserts")
