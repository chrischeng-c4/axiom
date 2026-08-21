# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "behavior"
# case = "test_tty__test_setcbreak"
# subject = "cpython.test_tty.TestTty.test_setcbreak"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""TestTty.test_setcbreak: tty.setcbreak returns and applies cbreak mode like CPython."""

import os
import termios
import tty


def check_cbreak(mode):
    assert mode[3] & termios.ECHO == 0, mode
    assert mode[3] & termios.ICANON == 0, mode
    assert mode[6][termios.VMIN] == 1, mode
    assert mode[6][termios.VTIME] == 0, mode


if not hasattr(os, "openpty"):
    print("TestTty::test_setcbreak: skipped no openpty")
else:
    master_fd, slave_fd = os.openpty()
    stream = None
    original = None
    try:
        stream = open(slave_fd, "wb", buffering=0)
        fd = stream.fileno()
        original = termios.tcgetattr(fd)

        mode1 = tty.setcbreak(fd)
        assert mode1 == original, (mode1, original)
        mode2 = termios.tcgetattr(fd)
        check_cbreak(mode2)
        assert mode2[tty.IFLAG] & termios.ICRNL == original[tty.IFLAG] & termios.ICRNL, (
            mode2,
            original,
        )

        mode3 = tty.setcbreak(fd, termios.TCSANOW)
        assert mode3 == mode2, (mode3, mode2)
        tty.setcbreak(stream)
        tty.setcbreak(fd=fd, when=termios.TCSANOW)
    finally:
        if stream is not None:
            if original is not None:
                termios.tcsetattr(stream.fileno(), termios.TCSANOW, original)
                termios.tcsetattr(stream.fileno(), termios.TCSAFLUSH, original)
            stream.close()
        os.close(master_fd)

    print("TestTty::test_setcbreak: ok")
