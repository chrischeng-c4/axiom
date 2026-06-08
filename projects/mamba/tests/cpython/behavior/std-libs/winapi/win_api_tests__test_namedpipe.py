# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winapi"
# dimension = "behavior"
# case = "win_api_tests__test_namedpipe"
# subject = "cpython.test_winapi.WinAPITests.test_namedpipe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_winapi.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_winapi.py::WinAPITests::test_namedpipe
"""Auto-ported test: WinAPITests::test_namedpipe."""


import sys


if sys.platform != "win32":
    print("WinAPITests::test_namedpipe: skipped Windows only")
else:
    import _winapi

    pipe_name = r"\\.\pipe\LOCAL\mamba-cpython-winapi-test"
    try:
        _winapi.WaitNamedPipe(pipe_name, 0)
    except FileNotFoundError:
        pass
    else:
        raise AssertionError("missing pipe should raise FileNotFoundError")

    pipe = _winapi.CreateNamedPipe(
        pipe_name,
        _winapi.PIPE_ACCESS_DUPLEX,
        8,
        2,
        32,
        32,
        0,
        0,
    )
    try:
        _winapi.WaitNamedPipe(pipe_name, 0)
        with open(pipe_name, "w+b") as pipe2:
            try:
                _winapi.WaitNamedPipe(pipe_name, 0)
            except OSError:
                pass
            else:
                raise AssertionError("unavailable pipe should raise OSError")

            _winapi.WriteFile(pipe, b"testdata")
            assert pipe2.read(8) == b"testdata"
            assert _winapi.PeekNamedPipe(pipe, 8)[:2] == (b"", 0)
            pipe2.write(b"testdata")
            pipe2.flush()
            assert _winapi.PeekNamedPipe(pipe, 8)[:2] == (b"testdata", 8)
    finally:
        _winapi.CloseHandle(pipe)

    print("WinAPITests::test_namedpipe: ok")
