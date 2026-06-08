# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "makefile_returns_readable_file"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: makefile('rb', buffering=0) is readable but not writable/seekable, raises ValueError on capability queries after close, and a closed socket-file reprs with name=-1"""
import socket

# An unbuffered read file is readable but not writable or seekable.
with socket.socket() as sock:
    fp = sock.makefile("rb", buffering=0)
    assert fp.readable(), "rb file should be readable"
    assert not fp.writable(), "rb file should not be writable"
    assert not fp.seekable(), "socket file should not be seekable"

    # Once closed, the capability queries raise ValueError.
    fp.close()
    for method in ("readable", "writable", "seekable"):
        raised = False
        try:
            getattr(fp, method)()
        except ValueError:
            raised = True
        assert raised, f"{method}() on closed file should raise ValueError"

# A closed socket-file reports a -1 file descriptor in its repr.
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    fp = sock.makefile("rb")
    fp.close()
    assert repr(fp) == "<_io.BufferedReader name=-1>", repr(fp)
print("makefile_returns_readable_file OK")
