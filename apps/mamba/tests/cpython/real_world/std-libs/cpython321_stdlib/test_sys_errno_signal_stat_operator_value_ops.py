# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_sys_errno_signal_stat_operator_value_ops"
# subject = "cpython321.test_sys_errno_signal_stat_operator_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_sys_errno_signal_stat_operator_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_sys_errno_signal_stat_operator_value_ops: execute CPython 3.12 seed test_sys_errno_signal_stat_operator_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of five
# bootstrap stdlib modules used by every CLI / signal-handler /
# filesystem-mode / functional-style script: `sys` (the
# byteorder / argv / modules / path / executable surface plus the
# `sys` self-import sentinel), `errno` (the documented POSIX
# integer error codes shared between macOS and Linux), `signal`
# (the documented portable signal integer sentinels and SIG_DFL
# / SIG_IGN / NSIG handler-disposition constants), `stat` (the
# full S_IF* file-type byte sentinels + S_I{R,W,X}{USR,GRP,OTH}
# permission-mode integer sentinels + S_ISDIR / S_ISREG mode-
# predicate helpers), and `operator` (the full set of documented
# arithmetic / comparison / logical / bitwise / sequence-arity
# primitives that mamba lowers to JIT-native i64 ops).
#
# The matching subset between mamba and CPython is the byte-exact
# constant + lossless-call layer: sys.byteorder == "little";
# sys.path / sys.modules / sys.executable / sys.argv carry the
# documented types; "sys" in sys.modules; errno integer codes
# match the POSIX values shared between Darwin and Linux (ENOENT
# == 2, EACCES == 13, etc.); signal SIGINT / SIGTERM / SIGKILL /
# SIGABRT integer values match; signal.SIG_DFL == 0, SIG_IGN == 1,
# NSIG == 32; stat S_IF*/S_I*/S_ISUID/S_ISGID/S_ISVTX integer
# sentinels match; stat.S_ISDIR(16384) and S_ISREG(32768) return
# True; operator.add/sub/mul/truediv/floordiv/mod/pow/neg/eq/ne/
# lt/le/gt/ge/contains/not_/truth/is_/is_not/and_/or_/xor/lshift/
# rshift/inv/abs/concat/countOf/indexOf all return the documented
# values.
#
# Surface in this fixture:
#   • sys.byteorder == "little" — POSIX little-endian sentinel;
#   • type(sys.path) is list — module search path;
#   • type(sys.modules) is dict — loaded-modules cache;
#   • "sys" in sys.modules — self-import sentinel;
#   • type(sys.executable) is str + non-empty;
#   • type(sys.argv) is list — CLI argument vector;
#   • hasattr(sys, "api_version") / "flags" / "float_info";
#   • errno.ENOENT == 2, EACCES == 13, EEXIST == 17, EISDIR == 21,
#     EINVAL == 22, EPIPE == 32, ENOTDIR == 20 — POSIX errno
#     codes shared between Darwin and Linux;
#   • type(errno.errorcode) is dict;
#   • signal.SIGINT == 2, SIGTERM == 15, SIGKILL == 9, SIGABRT == 6,
#     SIGUSR1 == 30, SIGUSR2 == 31 — POSIX integer sentinels;
#   • signal.SIG_DFL == 0, SIG_IGN == 1 — handler-disposition
#     sentinels;
#   • signal.NSIG == 32 — POSIX maximum-signal count;
#   • stat.S_IFREG == 32768, S_IFDIR == 16384, S_IFLNK == 40960
#     — file-type byte sentinels;
#   • stat.S_IFCHR == 8192, S_IFBLK == 24576, S_IFIFO == 4096,
#     S_IFSOCK == 49152;
#   • stat.S_IRUSR == 256, S_IWUSR == 128, S_IXUSR == 64;
#   • stat.S_IRGRP == 32, S_IWGRP == 16, S_IXGRP == 8;
#   • stat.S_IROTH == 4, S_IWOTH == 2, S_IXOTH == 1;
#   • stat.S_ISUID == 2048, S_ISGID == 1024, S_ISVTX == 512;
#   • stat.S_ISDIR(16384) is True, S_ISREG(32768) is True;
#   • operator.add(3, 4) == 7, sub(7, 2) == 5, mul(3, 5) == 15;
#   • operator.truediv(7, 2) == 3.5, floordiv(7, 2) == 3,
#     mod(7, 3) == 1, pow(2, 10) == 1024, neg(5) == -5;
#   • operator.eq / ne / lt / le / gt / ge return the documented
#     booleans;
#   • operator.contains([1,2,3], 2) is True, not_(False) is True,
#     truth(1) is True, is_(None, None) is True;
#   • operator.and_(5, 3) == 1, or_(5, 3) == 7, xor(5, 3) == 6;
#   • operator.lshift(1, 4) == 16, rshift(16, 2) == 4,
#     inv(0) == -1, abs(-5) == 5;
#   • operator.concat([1,2], [3,4]) == [1,2,3,4];
#   • operator.countOf([1,2,2,3], 2) == 2,
#     indexOf([10,20,30], 20) == 1.
#
# Behavioral edges that DIVERGE on mamba (sys.platform / maxsize /
# version_info / maxunicode / stdin / stdout / stderr; signal.
# Signals / Handlers class identity, type(SIGINT) is Signals,
# type(SIG_DFL) is Handlers; operator.attrgetter / methodcaller)
# are covered in `lang_sys_signal_operator_class_silent`.
import sys
import errno
import signal
import stat
import operator

_ledger: list[int] = []

# 1) sys — byteorder POSIX little-endian sentinel
assert sys.byteorder == "little"; _ledger.append(1)

# 2) sys — documented container types for path / modules / argv
assert type(sys.path).__name__ == "list"; _ledger.append(1)
assert type(sys.modules).__name__ == "dict"; _ledger.append(1)
assert type(sys.argv).__name__ == "list"; _ledger.append(1)

# 3) sys.executable — non-empty str
assert type(sys.executable).__name__ == "str"; _ledger.append(1)
assert len(sys.executable) > 0; _ledger.append(1)

# 4) sys.modules — self-import sentinel
assert "sys" in sys.modules; _ledger.append(1)

# 5) sys — documented introspection attribute surface
assert hasattr(sys, "api_version"); _ledger.append(1)
assert hasattr(sys, "flags"); _ledger.append(1)
assert hasattr(sys, "float_info"); _ledger.append(1)

# 6) errno — POSIX integer codes shared between Darwin and Linux
assert errno.ENOENT == 2; _ledger.append(1)
assert errno.EACCES == 13; _ledger.append(1)
assert errno.EEXIST == 17; _ledger.append(1)
assert errno.EISDIR == 21; _ledger.append(1)
assert errno.EINVAL == 22; _ledger.append(1)
assert errno.EPIPE == 32; _ledger.append(1)
assert errno.ENOTDIR == 20; _ledger.append(1)

# 7) errno.errorcode — documented dict
assert type(errno.errorcode).__name__ == "dict"; _ledger.append(1)

# 8) signal — POSIX integer sentinels (portable subset)
assert signal.SIGINT == 2; _ledger.append(1)
assert signal.SIGTERM == 15; _ledger.append(1)
assert signal.SIGKILL == 9; _ledger.append(1)
assert signal.SIGABRT == 6; _ledger.append(1)
assert signal.SIGUSR1 == 30; _ledger.append(1)
assert signal.SIGUSR2 == 31; _ledger.append(1)

# 9) signal — handler-disposition sentinels
assert signal.SIG_DFL == 0; _ledger.append(1)
assert signal.SIG_IGN == 1; _ledger.append(1)
assert signal.NSIG == 32; _ledger.append(1)

# 10) stat — file-type byte sentinels
assert stat.S_IFREG == 32768; _ledger.append(1)
assert stat.S_IFDIR == 16384; _ledger.append(1)
assert stat.S_IFLNK == 40960; _ledger.append(1)
assert stat.S_IFCHR == 8192; _ledger.append(1)
assert stat.S_IFBLK == 24576; _ledger.append(1)
assert stat.S_IFIFO == 4096; _ledger.append(1)
assert stat.S_IFSOCK == 49152; _ledger.append(1)

# 11) stat — owner / group / other permission-mode sentinels
assert stat.S_IRUSR == 256; _ledger.append(1)
assert stat.S_IWUSR == 128; _ledger.append(1)
assert stat.S_IXUSR == 64; _ledger.append(1)
assert stat.S_IRGRP == 32; _ledger.append(1)
assert stat.S_IWGRP == 16; _ledger.append(1)
assert stat.S_IXGRP == 8; _ledger.append(1)
assert stat.S_IROTH == 4; _ledger.append(1)
assert stat.S_IWOTH == 2; _ledger.append(1)
assert stat.S_IXOTH == 1; _ledger.append(1)

# 12) stat — setuid / setgid / sticky-bit sentinels
assert stat.S_ISUID == 2048; _ledger.append(1)
assert stat.S_ISGID == 1024; _ledger.append(1)
assert stat.S_ISVTX == 512; _ledger.append(1)

# 13) stat — mode-predicate helpers
assert stat.S_ISDIR(16384) == True; _ledger.append(1)
assert stat.S_ISREG(32768) == True; _ledger.append(1)

# 14) operator — arithmetic primitives
assert operator.add(3, 4) == 7; _ledger.append(1)
assert operator.sub(7, 2) == 5; _ledger.append(1)
assert operator.mul(3, 5) == 15; _ledger.append(1)
assert operator.truediv(7, 2) == 3.5; _ledger.append(1)
assert operator.floordiv(7, 2) == 3; _ledger.append(1)
assert operator.mod(7, 3) == 1; _ledger.append(1)
assert operator.pow(2, 10) == 1024; _ledger.append(1)
assert operator.neg(5) == -5; _ledger.append(1)
assert operator.abs(-5) == 5; _ledger.append(1)

# 15) operator — comparison primitives
assert operator.eq(3, 3) == True; _ledger.append(1)
assert operator.ne(3, 4) == True; _ledger.append(1)
assert operator.lt(1, 2) == True; _ledger.append(1)
assert operator.le(2, 2) == True; _ledger.append(1)
assert operator.gt(3, 2) == True; _ledger.append(1)
assert operator.ge(3, 3) == True; _ledger.append(1)

# 16) operator — logical / identity primitives
assert operator.not_(False) == True; _ledger.append(1)
assert operator.truth(1) == True; _ledger.append(1)
assert operator.is_(None, None) == True; _ledger.append(1)
assert operator.is_not(None, 1) == True; _ledger.append(1)
assert operator.contains([1, 2, 3], 2) == True; _ledger.append(1)

# 17) operator — bitwise primitives
assert operator.and_(5, 3) == 1; _ledger.append(1)
assert operator.or_(5, 3) == 7; _ledger.append(1)
assert operator.xor(5, 3) == 6; _ledger.append(1)
assert operator.lshift(1, 4) == 16; _ledger.append(1)
assert operator.rshift(16, 2) == 4; _ledger.append(1)
assert operator.inv(0) == -1; _ledger.append(1)

# 18) operator — sequence-arity primitives
assert operator.concat([1, 2], [3, 4]) == [1, 2, 3, 4]; _ledger.append(1)
assert operator.countOf([1, 2, 2, 3], 2) == 2; _ledger.append(1)
assert operator.indexOf([10, 20, 30], 20) == 1; _ledger.append(1)

# 19) hasattr surface — module-level helpers
assert hasattr(errno, "ENOENT"); _ledger.append(1)
assert hasattr(signal, "SIGINT"); _ledger.append(1)
assert hasattr(stat, "S_IFREG"); _ledger.append(1)
assert hasattr(operator, "add"); _ledger.append(1)
assert hasattr(operator, "itemgetter"); _ledger.append(1)

# NB: sys.platform / maxsize / version_info / maxunicode, sys.
# stdin / stdout / stderr, signal.Signals / Handlers class
# identity, type(signal.SIGINT) is Signals / type(SIG_DFL) is
# Handlers, operator.attrgetter / methodcaller all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_sys_errno_signal_stat_operator_value_ops {sum(_ledger)} asserts")
