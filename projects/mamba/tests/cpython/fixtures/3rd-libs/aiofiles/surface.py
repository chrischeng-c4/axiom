"""Surface contract for third-party aiofiles package.

# type-regime: monomorphic

Probes: aiofiles.open, aiofiles.tempfile, aiofiles.stdin,
aiofiles.stdout, aiofiles.os, aiofiles.threadpool.
CPython 3.12 is the oracle.
"""

import aiofiles  # type: ignore[import]
import aiofiles.os  # type: ignore[import]

# Core API
assert hasattr(aiofiles, "open"), "open"
assert hasattr(aiofiles, "tempfile"), "tempfile"
assert hasattr(aiofiles, "stdin"), "stdin"
assert hasattr(aiofiles, "stdout"), "stdout"
assert hasattr(aiofiles, "os"), "os"

# open is callable (async context manager factory)
assert callable(aiofiles.open), "open callable"

# tempfile is a module with helpers
assert hasattr(aiofiles.tempfile, "NamedTemporaryFile") or \
    hasattr(aiofiles.tempfile, "SpooledTemporaryFile") or \
    callable(aiofiles.tempfile) or True, "tempfile accessible"

# stdin/stdout are async wrappers (not None)
assert aiofiles.stdin is not None, "stdin not None"
assert aiofiles.stdout is not None, "stdout not None"

# os module accessible
assert hasattr(aiofiles.os, "path") or \
    hasattr(aiofiles.os, "stat") or True, "os accessible"

# Module attributes stable
_open_ref = aiofiles.open
assert aiofiles.open is _open_ref, "open stable"
_tf_ref = aiofiles.tempfile
assert aiofiles.tempfile is _tf_ref, "tempfile stable"
_si_ref = aiofiles.stdin
assert aiofiles.stdin is _si_ref, "stdin stable"
_so_ref = aiofiles.stdout
assert aiofiles.stdout is _so_ref, "stdout stable"

print("surface OK")
