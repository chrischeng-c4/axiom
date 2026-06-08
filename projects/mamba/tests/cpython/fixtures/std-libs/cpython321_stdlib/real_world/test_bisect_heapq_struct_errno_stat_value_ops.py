# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_bisect_heapq_struct_errno_stat_value_ops"
# subject = "cpython321.test_bisect_heapq_struct_errno_stat_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bisect_heapq_struct_errno_stat_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_bisect_heapq_struct_errno_stat_value_ops: execute CPython 3.12 seed test_bisect_heapq_struct_errno_stat_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `bisect` / `heapq` / `struct` / `errno` / `stat` five-pack
# pinned to atomic 206: `bisect` (the documented full
# module-level helper identifier hasattr surface — `bisect` /
# `bisect_left` / `bisect_right` / `insort` / `insort_left` /
# `insort_right` + the documented binary-search /
# sorted-insert value contract — `bisect_left([1,3,5,7,9],4)
# == 2` / `bisect_right([1,3,5,7,9],5) == 3` /
# `bisect([1,3,5,7,9],4) == 2` / `insort([1,3,5,7,9],4) ->
# [1,3,4,5,7,9]`), `heapq` (the documented full module-level
# helper identifier hasattr surface — `heappush` / `heappop`
# / `heappushpop` / `heapreplace` / `heapify` / `merge` /
# `nlargest` / `nsmallest` + the documented min-heap
# state / pop / nlargest / nsmallest value contract),
# `struct` (the documented full module-level helper /
# class / exception identifier hasattr surface — `pack` /
# `unpack` / `pack_into` / `unpack_from` / `calcsize` /
# `iter_unpack` / `Struct` / `error` + the documented
# `pack(">i", 42).hex() == "0000002a"` / `calcsize(">i")
# == 4` / `calcsize(">bhi") == 7` / `unpack(">i",
# pack(">i", 42))[0] == 42` value contract), `errno`
# (the documented full module-level POSIX errno
# integer-constant identifier hasattr surface — `EAGAIN`
# / `EBADF` / `EBUSY` / `EEXIST` / `EINTR` / `EINVAL` /
# `EIO` / `EISDIR` / `EMFILE` / `ENOENT` / `ENOMEM` /
# `ENOSPC` / `EPERM` / `EPIPE` / `ESRCH` / `ETIMEDOUT` /
# `errorcode` + the documented `type(errno.EAGAIN)
# .__name__ == "int"` / `errno.EAGAIN > 0` /
# `type(errno.errorcode).__name__ == "dict"`
# integer-constant value contract), and `stat` (the
# documented full module-level mode-test / mode-mask /
# permission-bit / index-constant identifier hasattr
# surface — `S_ISDIR` / `S_ISREG` / `S_ISLNK` /
# `S_ISCHR` / `S_ISBLK` / `S_ISFIFO` / `S_ISSOCK` /
# `S_IFMT` / `S_IFDIR` / `S_IFREG` / `S_IFLNK` /
# `S_IRUSR` / `S_IWUSR` / `S_IXUSR` / `S_IRGRP` /
# `S_IWGRP` / `S_IXGRP` / `S_IROTH` / `S_IWOTH` /
# `S_IXOTH` / `filemode` / `ST_MODE` / `ST_INO` /
# `ST_DEV` / `ST_NLINK` / `ST_UID` / `ST_GID` /
# `ST_SIZE` / `ST_ATIME` / `ST_MTIME` / `ST_CTIME` +
# the documented `type(stat.ST_MODE).__name__ ==
# "int"` / `type(stat.S_IRUSR).__name__ == "int"`
# integer-constant value contract).
#
# Behavioral edges that DIVERGE on mamba
# (the full `array.array` instance method surface —
# `append` / `buffer_info` / `byteswap` / `count` /
# `extend` / `frombytes` / `fromlist` / `index` /
# `insert` / `pop` / `remove` / `reverse` / `tobytes` /
# `tolist` / `typecode` / `itemsize` all False on
# mamba + `type(array.array("i", [...])).__name__`
# collapses to "int" on mamba instead of "array",
# `hasattr(codecs, "unregister")` / `"CodecInfo"` /
# `"make_identity_dict"` / `"make_encoding_map"` all
# False on mamba + `codecs.encode("abc", "rot13") ==
# "nop"` False on mamba, `hasattr(io, "TextIOWrapper")
# / "BufferedReader" / "BufferedWriter" /
# "BufferedRandom" / "FileIO" / "RawIOBase" /
# "BufferedIOBase" / "TextIOBase" / "IOBase" / "open" /
# "UnsupportedOperation" / "DEFAULT_BUFFER_SIZE" /
# "SEEK_SET" / "SEEK_CUR" / "SEEK_END"` all False on
# mamba + `type(io.StringIO()).__name__ == "StringIO"
# / "BytesIO"` collapses to "dict" on mamba) are
# covered in the matching spec fixture
# `lang_array_codecs_io_silent`.
import bisect
import heapq
import struct
import errno
import stat


_ledger: list[int] = []

# 1) bisect — full module hasattr surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 2) bisect — binary-search / sorted-insert value contract
_arr = [1, 3, 5, 7, 9]
assert bisect.bisect_left(_arr, 4) == 2; _ledger.append(1)
assert bisect.bisect_right(_arr, 5) == 3; _ledger.append(1)
assert bisect.bisect(_arr, 4) == 2; _ledger.append(1)
_arr2 = [1, 3, 5, 7, 9]
bisect.insort(_arr2, 4)
assert _arr2 == [1, 3, 4, 5, 7, 9]; _ledger.append(1)

# 3) heapq — full module hasattr surface
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)

# 4) heapq — min-heap value contract
_h: list[int] = []
heapq.heappush(_h, 3)
heapq.heappush(_h, 1)
heapq.heappush(_h, 2)
assert _h == [1, 3, 2]; _ledger.append(1)
assert heapq.heappop(_h) == 1; _ledger.append(1)
assert heapq.nlargest(3, [5, 1, 3, 7, 2, 8, 4]) == [8, 7, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [5, 1, 3, 7, 2, 8, 4]) == [1, 2, 3]; _ledger.append(1)

# 5) struct — full module hasattr surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)

# 6) struct — pack/unpack/calcsize value contract
assert struct.pack(">i", 42).hex() == "0000002a"; _ledger.append(1)
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">bhi") == 7; _ledger.append(1)
_u = struct.unpack(">i", struct.pack(">i", 42))
assert type(_u).__name__ == "tuple"; _ledger.append(1)
assert _u[0] == 42; _ledger.append(1)

# 7) errno — full POSIX errno integer-constant identifier surface
assert hasattr(errno, "EAGAIN") == True; _ledger.append(1)
assert hasattr(errno, "EBADF") == True; _ledger.append(1)
assert hasattr(errno, "EBUSY") == True; _ledger.append(1)
assert hasattr(errno, "EEXIST") == True; _ledger.append(1)
assert hasattr(errno, "EINTR") == True; _ledger.append(1)
assert hasattr(errno, "EINVAL") == True; _ledger.append(1)
assert hasattr(errno, "EIO") == True; _ledger.append(1)
assert hasattr(errno, "EISDIR") == True; _ledger.append(1)
assert hasattr(errno, "EMFILE") == True; _ledger.append(1)
assert hasattr(errno, "ENOENT") == True; _ledger.append(1)
assert hasattr(errno, "ENOMEM") == True; _ledger.append(1)
assert hasattr(errno, "ENOSPC") == True; _ledger.append(1)
assert hasattr(errno, "EPERM") == True; _ledger.append(1)
assert hasattr(errno, "EPIPE") == True; _ledger.append(1)
assert hasattr(errno, "ESRCH") == True; _ledger.append(1)
assert hasattr(errno, "ETIMEDOUT") == True; _ledger.append(1)
assert hasattr(errno, "errorcode") == True; _ledger.append(1)

# 8) errno — integer-constant value contract
assert type(errno.EAGAIN).__name__ == "int"; _ledger.append(1)
assert errno.EAGAIN > 0; _ledger.append(1)
assert type(errno.EBADF).__name__ == "int"; _ledger.append(1)
assert type(errno.ENOENT).__name__ == "int"; _ledger.append(1)
assert type(errno.errorcode).__name__ == "dict"; _ledger.append(1)

# 9) stat — full module hasattr surface
assert hasattr(stat, "S_ISDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISREG") == True; _ledger.append(1)
assert hasattr(stat, "S_ISLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_ISCHR") == True; _ledger.append(1)
assert hasattr(stat, "S_ISBLK") == True; _ledger.append(1)
assert hasattr(stat, "S_ISFIFO") == True; _ledger.append(1)
assert hasattr(stat, "S_ISSOCK") == True; _ledger.append(1)
assert hasattr(stat, "S_IFMT") == True; _ledger.append(1)
assert hasattr(stat, "S_IFDIR") == True; _ledger.append(1)
assert hasattr(stat, "S_IFREG") == True; _ledger.append(1)
assert hasattr(stat, "S_IFLNK") == True; _ledger.append(1)
assert hasattr(stat, "S_IRUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IWUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IXUSR") == True; _ledger.append(1)
assert hasattr(stat, "S_IRGRP") == True; _ledger.append(1)
assert hasattr(stat, "S_IWGRP") == True; _ledger.append(1)
assert hasattr(stat, "S_IXGRP") == True; _ledger.append(1)
assert hasattr(stat, "S_IROTH") == True; _ledger.append(1)
assert hasattr(stat, "S_IWOTH") == True; _ledger.append(1)
assert hasattr(stat, "S_IXOTH") == True; _ledger.append(1)
assert hasattr(stat, "filemode") == True; _ledger.append(1)
assert hasattr(stat, "ST_MODE") == True; _ledger.append(1)
assert hasattr(stat, "ST_INO") == True; _ledger.append(1)
assert hasattr(stat, "ST_DEV") == True; _ledger.append(1)
assert hasattr(stat, "ST_NLINK") == True; _ledger.append(1)
assert hasattr(stat, "ST_UID") == True; _ledger.append(1)
assert hasattr(stat, "ST_GID") == True; _ledger.append(1)
assert hasattr(stat, "ST_SIZE") == True; _ledger.append(1)
assert hasattr(stat, "ST_ATIME") == True; _ledger.append(1)
assert hasattr(stat, "ST_MTIME") == True; _ledger.append(1)
assert hasattr(stat, "ST_CTIME") == True; _ledger.append(1)

# 10) stat — integer-constant value contract
assert type(stat.ST_MODE).__name__ == "int"; _ledger.append(1)
assert type(stat.S_IRUSR).__name__ == "int"; _ledger.append(1)

# NB: the full `array.array` instance method surface
# (`append` / `buffer_info` / `byteswap` / `count` /
# `extend` / `frombytes` / `fromlist` / `index` /
# `insert` / `pop` / `remove` / `reverse` / `tobytes` /
# `tolist` / `typecode` / `itemsize`) all False on
# mamba + `type(array.array("i", [...])).__name__`
# collapses to "int" on mamba instead of "array",
# `hasattr(codecs, "unregister")` / `"CodecInfo"` /
# `"make_identity_dict"` / `"make_encoding_map"` all
# False on mamba + `codecs.encode("abc", "rot13") ==
# "nop"` False on mamba, `hasattr(io, "TextIOWrapper")
# / "BufferedReader" / "BufferedWriter" /
# "BufferedRandom" / "FileIO" / "RawIOBase" /
# "BufferedIOBase" / "TextIOBase" / "IOBase" / "open" /
# "UnsupportedOperation" / "DEFAULT_BUFFER_SIZE" /
# "SEEK_SET" / "SEEK_CUR" / "SEEK_END"` all False on
# mamba + `type(io.StringIO()).__name__ == "StringIO"
# / "BytesIO"` collapses to "dict" on mamba — all
# DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_bisect_heapq_struct_errno_stat_value_ops {sum(_ledger)} asserts")
