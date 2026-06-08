# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pwd"
# dimension = "behavior"
# case = "pwd_test__test_errors"
# subject = "cpython.test_pwd.PwdTest.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pwd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: PwdTest::test_errors (CPython 3.12 oracle)."""

import pwd
import sys


def expect(exc_type, func, *args, contains=None):
    try:
        func(*args)
    except exc_type as exc:
        if contains is not None:
            assert contains in str(exc), str(exc)
    else:
        raise AssertionError(f"expected {exc_type.__name__} from {func.__name__}")


expect(TypeError, pwd.getpwuid)
expect(TypeError, pwd.getpwuid, 3.14)
expect(TypeError, pwd.getpwnam)
expect(TypeError, pwd.getpwnam, 42)
expect(TypeError, pwd.getpwall, 42)
expect(ValueError, pwd.getpwnam, "a\0b", contains="null")

by_name = {}
by_uid = {}
for name, _passwd, uid, _gid, _gecos, _dir, _shell in pwd.getpwall():
    by_name[name] = uid
    by_uid[uid] = name

all_names = list(by_name)
name_index = 0
fake_name = all_names[name_index] if all_names else "invaliduser"
while fake_name in by_name:
    chars = list(fake_name)
    for index in range(len(chars)):
        if chars[index] == "z":
            chars[index] = "A"
            break
        if chars[index] == "Z":
            continue
        chars[index] = chr(ord(chars[index]) + 1)
        break
    else:
        name_index += 1
        try:
            fake_name = all_names[name_index]
        except IndexError:
            break
    fake_name = "".join(chars)

expect(KeyError, pwd.getpwnam, fake_name)

fake_uid = sys.maxsize
assert fake_uid not in by_uid
expect(KeyError, pwd.getpwuid, fake_uid)
expect(KeyError, pwd.getpwuid, -1)
expect(KeyError, pwd.getpwuid, 2**128)
expect(KeyError, pwd.getpwuid, -(2**128))

print("PwdTest::test_errors: ok")
