# Operational AssertionPass seed for `tempfile.gettempdir`.
# Surface: gettempdir returns a non-empty string path that exists.
# Companion to stub/test_tempfile.py — vendored unittest seed.
import tempfile
import os
_ledger: list[int] = []
d = tempfile.gettempdir()
assert isinstance(d, str); _ledger.append(1)
assert len(d) > 0; _ledger.append(1)
assert d.startswith("/"); _ledger.append(1)
assert os.path.exists(d); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_tempfile_ops {sum(_ledger)} asserts")
