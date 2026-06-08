# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "security"
# case = "non_ascii_error_report_survives_ascii_stream"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: a source file with a non-ASCII byte literal triggers a SyntaxError whose report is written through an ASCII-only stdout stream; compile_dir returns False and the report names the SyntaxError with NO UnicodeEncodeError leaking through"""
import compileall
import contextlib
import io
import os
import tempfile

# Adversarial source: a bytes literal carrying a non-ASCII char. Compiling it
# must report a SyntaxError, and the reporter must survive an ASCII-only output
# stream without leaking a UnicodeEncodeError of its own.
with tempfile.TemporaryDirectory() as d:
    with open(os.path.join(d, "bad_bytes.py"), "w", encoding="utf-8") as f:
        f.write('b"€"')
    buffer = io.TextIOWrapper(io.BytesIO(), encoding="ascii")
    with contextlib.redirect_stdout(buffer):
        compiled = compileall.compile_dir(d)
    buffer.seek(0)
    report = buffer.read()
    assert compiled is False, compiled
    assert "SyntaxError: bytes can only contain ASCII literal characters" in report, report
    assert "UnicodeEncodeError" not in report, report
print("non_ascii_error_report_survives_ascii_stream OK")
