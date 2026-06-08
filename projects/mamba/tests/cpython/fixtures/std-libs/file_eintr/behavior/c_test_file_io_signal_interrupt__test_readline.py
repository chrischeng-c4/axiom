# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file_eintr"
# dimension = "behavior"
# case = "c_test_file_io_signal_interrupt__test_readline"
# subject = "cpython.test_file_eintr.CTestFileIOSignalInterrupt.test_readline"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file_eintr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file_eintr.py::CTestFileIOSignalInterrupt::test_readline
"""Auto-ported test: CTestFileIOSignalInterrupt::test_readline (CPython oracle)."""


import os
import select
import signal
import subprocess
import sys
import time


if os.name != "posix":
    print("CTestFileIOSignalInterrupt::test_readline: skipped, posix only")
    raise SystemExit(0)

process = subprocess.Popen(
    [
        sys.executable,
        "-u",
        "-c",
        (
            "import signal, sys;"
            "signal.signal(signal.SIGINT, lambda s, f: sys.stderr.write('$\\n'));"
            "import _io as io;"
            "infile = io.FileIO(sys.stdin.fileno(), 'rb');"
            "sys.stderr.write('Worm Sign!\\n');"
            "got = infile.readline();"
            "expected = b'hello, world!\\n';"
            "assert got == expected, ('readline returned wrong data', got, expected);"
            "infile.close()"
        ),
    ],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
)

try:
    worm_sign = process.stderr.read(len(b"Worm Sign!\n"))
    if worm_sign != b"Worm Sign!\n":
        raise AssertionError(f"reader process did not become ready: {worm_sign!r}")

    assert process.stdin is not None
    process.stdin.write(b"hello, world!")
    process.stdin.flush()

    signals_sent = 0
    ready = []
    while not ready:
        ready, _, _ = select.select([process.stderr], (), (), 0.05)
        process.send_signal(signal.SIGINT)
        signals_sent += 1
        if signals_sent > 200:
            process.kill()
            raise AssertionError("reader process failed to handle signals")

    signal_line = process.stderr.readline()
    if signal_line != b"$\n":
        raise AssertionError(f"reader process emitted unexpected signal line: {signal_line!r}")

    stdout, stderr = process.communicate(input=b"\n", timeout=10)
    if process.returncode:
        raise AssertionError(
            f"reader process exited rc={process.returncode}\n"
            f"STDOUT:\n{stdout.decode(errors='replace')}"
            f"STDERR:\n{stderr.decode(errors='replace')}"
        )
finally:
    if process.poll() is None:
        try:
            process.terminate()
            process.wait(timeout=2)
        except OSError:
            pass
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=2)

print("CTestFileIOSignalInterrupt::test_readline: ok")
