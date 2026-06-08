# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "behavior"
# case = "test_tty__test_setraw"
# subject = "cpython.test_tty.TestTty.test_setraw"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""TestTty.test_setraw: tty.setraw returns and applies raw mode like CPython."""

import os
import termios
import tty


def check_raw(mode):
    assert mode[3] & termios.ECHO == 0, mode
    assert mode[3] & termios.ICANON == 0, mode
    assert mode[6][termios.VMIN] == 1, mode
    assert mode[6][termios.VTIME] == 0, mode
    assert mode[0] & termios.ISTRIP == 0, mode
    assert mode[0] & termios.ICRNL == 0, mode
    assert mode[1] & termios.OPOST == 0, mode
    assert mode[2] & termios.PARENB == termios.CS8 & termios.PARENB, mode
    assert mode[2] & termios.CSIZE == termios.CS8 & termios.CSIZE, mode
    assert mode[2] & termios.CS8 == termios.CS8, mode
    assert mode[3] & termios.ISIG == 0, mode


if not hasattr(os, "openpty"):
    print("TestTty::test_setraw: skipped no openpty")
else:
    master_fd, slave_fd = os.openpty()
    stream = None
    original = None
    try:
        stream = open(slave_fd, "wb", buffering=0)
        fd = stream.fileno()
        original = termios.tcgetattr(fd)

        mode1 = tty.setraw(fd)
        assert mode1 == original, (mode1, original)
        mode2 = termios.tcgetattr(fd)
        check_raw(mode2)

        mode3 = tty.setraw(fd, termios.TCSANOW)
        assert mode3 == mode2, (mode3, mode2)
        tty.setraw(stream)
        tty.setraw(fd=fd, when=termios.TCSANOW)
    finally:
        if stream is not None:
            if original is not None:
                termios.tcsetattr(stream.fileno(), termios.TCSANOW, original)
                termios.tcsetattr(stream.fileno(), termios.TCSAFLUSH, original)
            stream.close()
        os.close(master_fd)

    print("TestTty::test_setraw: ok")
