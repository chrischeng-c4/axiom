# Operational AssertionPass seed for `os` path-format constants.
# Surface: sep, linesep, curdir, pardir, extsep, devnull at their
# canonical POSIX values. These are part of the documented os module
# API used by portable path-string construction.
# Companion to stub/test_os.py — vendored unittest seed.
import os
_ledger: list[int] = []
# POSIX path component separators
assert os.sep == "/"; _ledger.append(1)
assert os.curdir == "."; _ledger.append(1)
assert os.pardir == ".."; _ledger.append(1)
assert os.extsep == "."; _ledger.append(1)
# POSIX line terminator + null device
assert os.linesep == "\n"; _ledger.append(1)
assert os.devnull == "/dev/null"; _ledger.append(1)
# Length / type invariants
assert len(os.sep) == 1; _ledger.append(1)
assert len(os.pardir) == 2; _ledger.append(1)
assert isinstance(os.sep, str); _ledger.append(1)
assert isinstance(os.devnull, str); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_os_constants_ops {sum(_ledger)} asserts")
