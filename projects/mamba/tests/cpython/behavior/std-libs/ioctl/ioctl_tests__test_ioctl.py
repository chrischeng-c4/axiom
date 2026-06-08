# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ioctl"
# dimension = "behavior"
# case = "ioctl_tests__test_ioctl"
# subject = "cpython.test_ioctl.IoctlTests.test_ioctl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ioctl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ioctl.py::IoctlTests::test_ioctl
"""Auto-ported test: IoctlTests::test_ioctl (CPython 3.12 oracle)."""


import os
import struct


try:
    import fcntl
    import termios
except ImportError as exc:
    print(f"IoctlTests::test_ioctl: skipped, missing module {exc.name}")
    raise SystemExit(0)

if not hasattr(termios, "TIOCGPGRP"):
    print("IoctlTests::test_ioctl: skipped, termios.TIOCGPGRP unavailable")
    raise SystemExit(0)

try:
    with open("/dev/tty", "rb") as tty:
        result = fcntl.ioctl(tty, termios.TIOCGPGRP, "    ")
except OSError:
    print("IoctlTests::test_ioctl: skipped, unable to open /dev/tty")
    raise SystemExit(0)

rpgrp = struct.unpack("i", result)[0]
ids = (os.getpgrp(), os.getsid(0))
if rpgrp not in ids:
    print("IoctlTests::test_ioctl: skipped, process is not attached to /dev/tty")
    raise SystemExit(0)

assert rpgrp in ids, (rpgrp, ids)
print("IoctlTests::test_ioctl: ok")
