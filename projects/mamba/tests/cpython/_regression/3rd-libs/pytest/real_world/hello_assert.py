"""Import pytest and drive one passing assertion through `pytest.main`.

End-user scenario: a downstream test suite needs pytest to discover
and execute a single trivial test function and report success. This
is the smallest reproducible "pytest runs unchanged" gate — anything
beyond a single in-process test invocation belongs in a larger
fixture.

DoD: this script must exit 0 under both CPython and mamba.
"""

import os
import tempfile
import pytest

# Inline the test source: pytest's collector wants an importable file
# on disk, so we write one to a TemporaryDirectory and point pytest at
# it. No network, no shared filesystem mutation outside the temp dir.
_TEST_SOURCE = """\
def test_hello_assert_passes():
    assert 1 + 1 == 2
"""

with tempfile.TemporaryDirectory() as tmp:
    test_path = os.path.join(tmp, "test_hello.py")
    with open(test_path, "w", encoding="ascii") as fh:
        fh.write(_TEST_SOURCE)

    # `pytest.main` returns ExitCode.OK (0) on green; -q keeps the
    # captured output short and stable.
    exit_code = pytest.main(["-q", test_path])

assert int(exit_code) == 0, f"pytest reported non-zero exit: {exit_code!r}"

print("ok:", int(exit_code))
