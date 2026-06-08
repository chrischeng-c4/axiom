# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dtrace"
# dimension = "behavior"
# case = "check_dtrace_probes__test_check_probes"
# subject = "cpython.test_dtrace.CheckDtraceProbes.test_check_probes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dtrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dtrace.py::CheckDtraceProbes::test_check_probes
"""Auto-ported test: CheckDtraceProbes::test_check_probes (CPython 3.12 oracle)."""


import re
import subprocess
import sys
import sysconfig


if not sysconfig.get_config_var("WITH_DTRACE"):
    print("CheckDtraceProbes::test_check_probes: skipped, CPython not built with dtrace")
    raise SystemExit(0)

try:
    proc = subprocess.Popen(
        ["readelf", "--version"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        universal_newlines=True,
    )
    version, stderr = proc.communicate()
except OSError:
    print("CheckDtraceProbes::test_check_probes: skipped, readelf unavailable")
    raise SystemExit(0)

if proc.returncode:
    raise AssertionError(f"readelf --version failed: stdout={version!r} stderr={stderr!r}")
if re.search(r"^(?:GNU) readelf.*?\b(\d+)\.(\d+)", version) is None:
    print(f"CheckDtraceProbes::test_check_probes: skipped, unable to parse readelf version: {version!r}")
    raise SystemExit(0)

readelf = subprocess.Popen(
    ["readelf", "-n", sys.executable],
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    universal_newlines=True,
)
readelf_output, _ = readelf.communicate()

for probe_name in (
    "Name: import__find__load__done",
    "Name: import__find__load__start",
    "Name: audit",
    "Name: gc__start",
    "Name: gc__done",
):
    assert probe_name in readelf_output, probe_name

print("CheckDtraceProbes::test_check_probes: ok")
