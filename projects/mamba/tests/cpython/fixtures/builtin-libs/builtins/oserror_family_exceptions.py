# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Eleven exception classes that were previously undefined-name in
# mamba's type-checker — they now register as classes and behave as
# CPython exception subclasses (#1324).

# OSError-family (parent OSError directly):
try:
    raise BlockingIOError("blocked")
except OSError:
    print("BlockingIOError -> OSError")

try:
    raise ChildProcessError("child gone")
except OSError:
    print("ChildProcessError -> OSError")

try:
    raise InterruptedError("EINTR")
except OSError:
    print("InterruptedError -> OSError")

try:
    raise ProcessLookupError("ESRCH")
except OSError:
    print("ProcessLookupError -> OSError")

try:
    raise IsADirectoryError("EISDIR")
except OSError:
    print("IsADirectoryError -> OSError")

try:
    raise NotADirectoryError("ENOTDIR")
except OSError:
    print("NotADirectoryError -> OSError")

# ConnectionError-family (parent ConnectionError, then OSError):
try:
    raise BrokenPipeError("EPIPE")
except ConnectionError:
    print("BrokenPipeError -> ConnectionError")

try:
    raise ConnectionAbortedError("ECONNABORTED")
except OSError:
    print("ConnectionAbortedError -> OSError")

try:
    raise ConnectionRefusedError("ECONNREFUSED")
except ConnectionError:
    print("ConnectionRefusedError -> ConnectionError")

try:
    raise ConnectionResetError("ECONNRESET")
except OSError:
    print("ConnectionResetError -> OSError")

# Reference family — parent Exception directly:
try:
    raise ReferenceError("weakref dead")
except Exception:
    print("ReferenceError -> Exception")

# isinstance walks the MRO.
err = ConnectionResetError("rst")
print(isinstance(err, ConnectionError))    # True
print(isinstance(err, OSError))            # True
print(isinstance(err, Exception))          # True
