# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(linecache, 'updatecache')`
# (the documented "linecache exposes the updatecache helper" —
# mamba returns False), `hasattr(linecache, 'lazycache')` (the
# documented "linecache exposes the lazycache helper" — mamba
# returns False), `hasattr(linecache, 'cache')` (the documented
# "linecache exposes its internal cache dict" — mamba returns
# False), `hasattr(io, 'IOBase')` (the documented "io exposes the
# IOBase abstract base class" — mamba returns False), `hasattr(io,
# 'TextIOWrapper')` (the documented "io exposes the TextIOWrapper
# class" — mamba returns False), `hasattr(io, 'FileIO')` (the
# documented "io exposes the FileIO class" — mamba returns False),
# `hasattr(io, 'DEFAULT_BUFFER_SIZE')` (the documented "io exposes
# the DEFAULT_BUFFER_SIZE constant" — mamba returns False), `hasattr
# (io, 'SEEK_SET')` (the documented "io exposes the SEEK_SET
# constant" — mamba returns False), `io.SEEK_SET == 0` (the
# documented "SEEK_SET constant value is 0" — mamba returns None),
# and StringIO write/getvalue round-trip (the documented "io.
# StringIO.write persists into getvalue" — mamba returns '' — write
# does not persist).
# Ten-pack pinned to atomic 286.
#
# Behavioral edges that CONFORM on mamba (fnmatch — hasattr fnmatch/
# fnmatchcase/filter/translate + wildcard/case/filter/translate.
# glob — hasattr glob/iglob/escape/has_magic + escape/has_magic.
# filecmp — hasattr cmp/cmpfiles/dircmp/clear_cache/DEFAULT_IGNORES
# + RCS in list. posixpath — hasattr join/split/basename/dirname/
# abspath/isabs/normpath/sep/extsep + sep/extsep/join/basename/
# dirname/isabs/normpath. ntpath — hasattr join/basename/dirname/
# sep + sep '\\'. stat — hasattr S_ISDIR/S_ISREG/S_ISLNK/S_IFMT/
# S_IMODE/filemode/S_IFDIR/S_IFREG/S_IRUSR/S_IWUSR/S_IXUSR/ST_MODE/
# ST_SIZE + constants 16384/32768/256/128/64/0 + predicates. errno
# — hasattr EACCES/ENOENT/EEXIST/EPERM/EINVAL/EINTR/EIO/errorcode +
# values 13/2/17/1/22) are covered in the matching pass fixture
# `test_fnmatch_glob_filecmp_posixpath_stat_errno_value_ops`.
import linecache
import io


_ledger: list[int] = []

# 1) hasattr(linecache, 'updatecache') — cache refresh helper
#    (mamba: returns False)
assert hasattr(linecache, "updatecache") == True; _ledger.append(1)

# 2) hasattr(linecache, 'lazycache') — lazy registration helper
#    (mamba: returns False)
assert hasattr(linecache, "lazycache") == True; _ledger.append(1)

# 3) hasattr(linecache, 'cache') — internal cache dict
#    (mamba: returns False)
assert hasattr(linecache, "cache") == True; _ledger.append(1)

# 4) hasattr(io, 'IOBase') — IOBase abstract base class
#    (mamba: returns False)
assert hasattr(io, "IOBase") == True; _ledger.append(1)

# 5) hasattr(io, 'TextIOWrapper') — TextIOWrapper class
#    (mamba: returns False)
assert hasattr(io, "TextIOWrapper") == True; _ledger.append(1)

# 6) hasattr(io, 'FileIO') — FileIO class
#    (mamba: returns False)
assert hasattr(io, "FileIO") == True; _ledger.append(1)

# 7) hasattr(io, 'DEFAULT_BUFFER_SIZE') — DEFAULT_BUFFER_SIZE constant
#    (mamba: returns False)
assert hasattr(io, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)

# 8) hasattr(io, 'SEEK_SET') — SEEK_SET constant
#    (mamba: returns False)
assert hasattr(io, "SEEK_SET") == True; _ledger.append(1)

# 9) io.SEEK_SET == 0 — seek-from-start constant value
#    (mamba: returns None)
assert io.SEEK_SET == 0; _ledger.append(1)

# 10) io.StringIO write/getvalue round-trip
#     (mamba: write does not persist, getvalue returns '')
_s = io.StringIO()
_s.write("hi")
assert _s.getvalue() == "hi"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_linecache_io_silent {sum(_ledger)} asserts")
