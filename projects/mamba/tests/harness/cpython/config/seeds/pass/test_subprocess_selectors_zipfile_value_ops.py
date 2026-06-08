# Atomic 309 pass conformance — subprocess module (hasattr Popen/run/
# call/check_call/check_output/PIPE/STDOUT/DEVNULL/CalledProcessError/
# TimeoutExpired/SubprocessError/CompletedProcess + PIPE == -1 +
# STDOUT == -2 + DEVNULL == -3) + selectors module (hasattr Default
# Selector/SelectSelector/PollSelector/KqueueSelector/EVENT_READ/
# EVENT_WRITE/SelectorKey/BaseSelector + EVENT_READ == 1 + EVENT_WRITE
# == 2) + zipfile module (hasattr ZipFile/is_zipfile/ZIP_STORED/ZIP_
# DEFLATED + ZIP_STORED == 0 + ZIP_DEFLATED == 8) + tarfile module
# (hasattr open/is_tarfile) + html.parser module (hasattr HTMLParser/
# unescape) + contextvars module (hasattr ContextVar/Context/Token/
# copy_context).
# All asserts match between CPython 3.12 and mamba.
import subprocess
import selectors
import zipfile
import tarfile
from html import parser as html_parser
import contextvars


_ledger: list[int] = []

# 1) subprocess — hasattr core surface + PIPE/STDOUT/DEVNULL constants
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "SubprocessError") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)
assert subprocess.PIPE == -1; _ledger.append(1)
assert subprocess.STDOUT == -2; _ledger.append(1)
assert subprocess.DEVNULL == -3; _ledger.append(1)

# 2) selectors — hasattr core surface + EVENT constants
assert hasattr(selectors, "DefaultSelector") == True; _ledger.append(1)
assert hasattr(selectors, "SelectSelector") == True; _ledger.append(1)
assert hasattr(selectors, "PollSelector") == True; _ledger.append(1)
assert hasattr(selectors, "KqueueSelector") == True; _ledger.append(1)
assert hasattr(selectors, "EVENT_READ") == True; _ledger.append(1)
assert hasattr(selectors, "EVENT_WRITE") == True; _ledger.append(1)
assert hasattr(selectors, "SelectorKey") == True; _ledger.append(1)
assert hasattr(selectors, "BaseSelector") == True; _ledger.append(1)
assert selectors.EVENT_READ == 1; _ledger.append(1)
assert selectors.EVENT_WRITE == 2; _ledger.append(1)

# 3) zipfile — hasattr (conformant subset) + ZIP method constants
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)
assert zipfile.ZIP_STORED == 0; _ledger.append(1)
assert zipfile.ZIP_DEFLATED == 8; _ledger.append(1)

# 4) tarfile — hasattr (conformant subset)
assert hasattr(tarfile, "open") == True; _ledger.append(1)
assert hasattr(tarfile, "is_tarfile") == True; _ledger.append(1)

# 5) html.parser — hasattr core surface
assert hasattr(html_parser, "HTMLParser") == True; _ledger.append(1)
assert hasattr(html_parser, "unescape") == True; _ledger.append(1)

# 6) contextvars — hasattr core surface
assert hasattr(contextvars, "ContextVar") == True; _ledger.append(1)
assert hasattr(contextvars, "Context") == True; _ledger.append(1)
assert hasattr(contextvars, "Token") == True; _ledger.append(1)
assert hasattr(contextvars, "copy_context") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_subprocess_selectors_zipfile_value_ops {sum(_ledger)} asserts")
