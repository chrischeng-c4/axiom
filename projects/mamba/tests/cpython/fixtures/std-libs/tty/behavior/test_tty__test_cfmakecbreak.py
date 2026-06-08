# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "behavior"
# case = "test_tty__test_cfmakecbreak"
# subject = "cpython.test_tty.TestTty.test_cfmakecbreak"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tty.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""TestTty.test_cfmakecbreak: tty.cfmakecbreak mutates cbreak flags like CPython."""

import os
import termios
import tty


def check_cbreak(mode):
    assert mode[3] & termios.ECHO == 0, mode
    assert mode[3] & termios.ICANON == 0, mode
    assert mode[6][termios.VMIN] == 1, mode
    assert mode[6][termios.VTIME] == 0, mode


if not hasattr(os, "openpty"):
    print("TestTty::test_cfmakecbreak: skipped no openpty")
else:
    master_fd, slave_fd = os.openpty()
    stream = None
    try:
        stream = open(slave_fd, "wb", buffering=0)
        fd = stream.fileno()
        original = termios.tcgetattr(fd)
        mode = termios.tcgetattr(fd)
        assert mode == original, (mode, original)

        tty.cfmakecbreak(mode)
        check_cbreak(mode)
        assert mode[1] == original[1], mode
        assert mode[2] == original[2], mode
        assert mode[4] == original[4], mode
        assert mode[5] == original[5], mode

        mode[tty.IFLAG] |= termios.ICRNL
        tty.cfmakecbreak(mode)
        assert mode[tty.IFLAG] & termios.ICRNL == termios.ICRNL, mode

        mode[tty.IFLAG] &= ~termios.ICRNL
        tty.cfmakecbreak(mode)
        assert mode[tty.IFLAG] & termios.ICRNL == 0, mode
    finally:
        if stream is not None:
            termios.tcsetattr(stream.fileno(), termios.TCSANOW, original)
            termios.tcsetattr(stream.fileno(), termios.TCSAFLUSH, original)
            stream.close()
        os.close(master_fd)

    print("TestTty::test_cfmakecbreak: ok")
