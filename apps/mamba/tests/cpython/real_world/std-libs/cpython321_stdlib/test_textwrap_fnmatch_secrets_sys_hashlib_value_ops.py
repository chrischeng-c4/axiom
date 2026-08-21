# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_textwrap_fnmatch_secrets_sys_hashlib_value_ops"
# subject = "cpython321.test_textwrap_fnmatch_secrets_sys_hashlib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_textwrap_fnmatch_secrets_sys_hashlib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_textwrap_fnmatch_secrets_sys_hashlib_value_ops: execute CPython 3.12 seed test_textwrap_fnmatch_secrets_sys_hashlib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 234 pass conformance — textwrap / fnmatch / glob / shutil / secrets /
# pathlib / platform / sys / os.path / time / uuid / calendar / hashlib value
# ops + hasattr surface that match between CPython 3.12 and mamba.
import textwrap
import fnmatch
import glob
import shutil
import secrets
import pathlib
import platform
import sys
import os.path as ospath
import time
import uuid
import calendar
import hashlib


_ledger: list[int] = []

# 1) textwrap value ops
assert textwrap.dedent("    hello\n    world") == "hello\nworld"; _ledger.append(1)
assert textwrap.indent("a\nb\nc", "  ") == "  a\n  b\n  c"; _ledger.append(1)
assert textwrap.fill("the quick brown fox", 20) == "the quick brown fox"; _ledger.append(1)
assert textwrap.wrap("the quick brown fox", 10) == ["the quick", "brown fox"]; _ledger.append(1)
assert textwrap.shorten("hello there how are you", 12) == "hello [...]"; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)

# 2) fnmatch value ops (basic case-sensitive forms that match both)
assert fnmatch.fnmatch("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("foo.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("FOO.PY", "*.py") == False; _ledger.append(1)
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)
assert type(fnmatch.translate("*.py")).__name__ == "str"; _ledger.append(1)

# 3) glob surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)

# 4) shutil surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)

# 5) secrets value ops
assert type(secrets.randbelow(100)).__name__ == "int"; _ledger.append(1)
assert type(secrets.randbits(8)).__name__ == "int"; _ledger.append(1)
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert type(secrets.token_urlsafe(8)).__name__ == "str"; _ledger.append(1)
assert secrets.compare_digest("hello", "hello") == True; _ledger.append(1)
assert secrets.compare_digest("hello", "world") == False; _ledger.append(1)
assert (secrets.choice([1, 2, 3]) in [1, 2, 3]) == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)

# 6) pathlib surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)

# 7) platform surface
assert type(platform.system()).__name__ == "str"; _ledger.append(1)
assert type(platform.machine()).__name__ == "str"; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)

# 8) sys surface
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "getsizeof") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "flags") == True; _ledger.append(1)
assert hasattr(sys, "hexversion") == True; _ledger.append(1)
assert hasattr(sys, "api_version") == True; _ledger.append(1)

# 9) os.path conforming value ops
assert ospath.join("/a", "b", "c") == "/a/b/c"; _ledger.append(1)
assert ospath.split("/a/b/c.txt") == ("/a/b", "c.txt"); _ledger.append(1)
assert ospath.splitext("file.txt") == ("file", ".txt"); _ledger.append(1)
assert ospath.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert ospath.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)
assert hasattr(ospath, "exists") == True; _ledger.append(1)
assert hasattr(ospath, "isfile") == True; _ledger.append(1)
assert hasattr(ospath, "isdir") == True; _ledger.append(1)
assert hasattr(ospath, "getsize") == True; _ledger.append(1)
assert hasattr(ospath, "abspath") == True; _ledger.append(1)
assert hasattr(ospath, "realpath") == True; _ledger.append(1)
assert hasattr(ospath, "expanduser") == True; _ledger.append(1)

# 10) time surface — float-returning ops
assert type(time.time()).__name__ == "float"; _ledger.append(1)
assert type(time.monotonic()).__name__ == "float"; _ledger.append(1)
assert type(time.perf_counter()).__name__ == "float"; _ledger.append(1)
assert type(time.process_time()).__name__ == "float"; _ledger.append(1)
assert hasattr(time, "sleep") == True; _ledger.append(1)
assert hasattr(time, "strftime") == True; _ledger.append(1)
assert hasattr(time, "strptime") == True; _ledger.append(1)
assert hasattr(time, "localtime") == True; _ledger.append(1)
assert hasattr(time, "gmtime") == True; _ledger.append(1)
assert hasattr(time, "mktime") == True; _ledger.append(1)
assert hasattr(time, "asctime") == True; _ledger.append(1)
assert hasattr(time, "ctime") == True; _ledger.append(1)
assert hasattr(time, "clock_gettime") == True; _ledger.append(1)

# 11) uuid surface + value ops
assert len(str(uuid.uuid4())) == 36; _ledger.append(1)
assert str(uuid.UUID("12345678-1234-1234-1234-123456789abc")) == "12345678-1234-1234-1234-123456789abc"; _ledger.append(1)
assert uuid.UUID("12345678-1234-1234-1234-123456789abc").hex == "12345678123412341234123456789abc"; _ledger.append(1)
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)

# 12) calendar value ops + surface
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)
assert hasattr(calendar, "Calendar") == True; _ledger.append(1)
assert hasattr(calendar, "month") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.FRIDAY == 4; _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)

# 13) hashlib value ops + algorithm surface
assert hashlib.sha256(b"hello").hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert hashlib.sha1(b"hello").hexdigest() == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"; _ledger.append(1)
assert hashlib.md5(b"hello").hexdigest() == "5d41402abc4b2a76b9719d911017c592"; _ledger.append(1)
assert hashlib.sha256().digest_size == 32; _ledger.append(1)
assert hashlib.sha256().block_size == 64; _ledger.append(1)
assert hashlib.sha256().name == "sha256"; _ledger.append(1)
assert hasattr(hashlib, "algorithms_available") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2b") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)
assert hasattr(hashlib, "sha3_256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha3_512") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_textwrap_fnmatch_secrets_sys_hashlib_value_ops {sum(_ledger)} asserts")
