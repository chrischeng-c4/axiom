# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_platform_silent"
# subject = "cpython321.lang_argparse_platform_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_platform_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_platform_silent: execute CPython 3.12 seed lang_argparse_platform_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(argparse, 'Namespace')` (the
# documented "argparse exposes the Namespace class" — mamba returns
# False), `hasattr(argparse, 'Action')` (the documented "argparse
# exposes the Action base class" — mamba returns False), `hasattr(
# argparse, 'SUPPRESS')` (the documented "argparse exposes the
# SUPPRESS sentinel" — mamba returns False), `hasattr(argparse, '
# ArgumentError')` (the documented "argparse exposes the
# ArgumentError exception" — mamba returns False), `hasattr(argparse,
# 'FileType')` (the documented "argparse exposes the FileType
# converter" — mamba returns False), `type(argparse.ArgumentParser(
# prog='t')).__name__ == 'ArgumentParser'` (the documented
# "ArgumentParser() returns an ArgumentParser instance" — mamba
# returns 'dict' — constructor degrades to a plain dict),
# `argparse.ArgumentParser(prog='t').prog == 't'` (the documented
# "ArgumentParser stores the constructor prog kwarg" — mamba returns
# None — attribute resolves to None placeholder), `hasattr(getpass, '
# GetPassWarning')` (the documented "getpass exposes the
# GetPassWarning class" — mamba returns False), `hasattr(platform, '
# uname')` (the documented "platform exposes the uname function" —
# mamba returns False), and `platform.system() == 'Darwin'` (the
# documented "platform.system returns the canonical kernel/OS name
# from `uname -s`" — mamba returns 'macos' — non-canonical lowercase
# product name).
# Ten-pack pinned to atomic 300.
#
# Behavioral edges that CONFORM on mamba (signal — hasattr full +
# SIGTERM/SIGINT/SIGHUP/SIGKILL/SIGABRT/SIGUSR1/signal/SIG_DFL/SIG_
# IGN/Signals. sys — hasattr full + version/version_info/platform/
# maxsize/path/modules/argv/std*/byteorder/float_info/int_info/hash_
# info/getrecursionlimit/exit + type contracts + byteorder. platform —
# hasattr system/machine/python_version/release/processor/node/
# platform + python_version str. getpass — hasattr getuser/getpass +
# getuser str. argparse — hasattr ArgumentParser) are covered in the
# matching pass fixture `test_signal_sys_platform_getpass_value_ops`.
import argparse
import getpass
import platform


_ledger: list[int] = []

# 1) hasattr(argparse, 'Namespace') — Namespace class
#    (mamba: returns False)
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)

# 2) hasattr(argparse, 'Action') — Action base class
#    (mamba: returns False)
assert hasattr(argparse, "Action") == True; _ledger.append(1)

# 3) hasattr(argparse, 'SUPPRESS') — SUPPRESS sentinel
#    (mamba: returns False)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)

# 4) hasattr(argparse, 'ArgumentError') — ArgumentError exception
#    (mamba: returns False)
assert hasattr(argparse, "ArgumentError") == True; _ledger.append(1)

# 5) hasattr(argparse, 'FileType') — FileType converter
#    (mamba: returns False)
assert hasattr(argparse, "FileType") == True; _ledger.append(1)

# 6) type(argparse.ArgumentParser(prog='t')).__name__ == 'ArgumentParser' — ArgumentParser instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(argparse.ArgumentParser(prog="t")).__name__ == "ArgumentParser"; _ledger.append(1)

# 7) argparse.ArgumentParser(prog='t').prog == 't' — prog kwarg echo
#    (mamba: returns None — attribute resolves to None placeholder)
assert argparse.ArgumentParser(prog="t").prog == "t"; _ledger.append(1)

# 8) hasattr(getpass, 'GetPassWarning') — GetPassWarning class
#    (mamba: returns False)
assert hasattr(getpass, "GetPassWarning") == True; _ledger.append(1)

# 9) hasattr(platform, 'uname') — uname function
#    (mamba: returns False)
assert hasattr(platform, "uname") == True; _ledger.append(1)

# 10) platform.system() == 'Darwin' — canonical kernel/OS name
#     (mamba: returns 'macos' — non-canonical lowercase product name)
assert platform.system() == "Darwin"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_platform_silent {sum(_ledger)} asserts")
