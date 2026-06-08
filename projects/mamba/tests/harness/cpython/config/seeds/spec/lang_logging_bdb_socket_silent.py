# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(logging, 'Logger')` (the
# documented "logging exposes the Logger class" — mamba returns
# False), `hasattr(logging, 'StreamHandler')` (the documented "logging
# exposes the StreamHandler class" — mamba returns False), `hasattr(
# logging, 'NOTSET')` (the documented "logging exposes the NOTSET
# level constant" — mamba returns False), `type(logging.getLogger('x')
# ).__name__ == 'Logger'` (the documented "getLogger returns a Logger
# instance" — mamba returns 'dict' — constructor degrades to plain
# dict), `hasattr(logging, 'addLevelName')` (the documented "logging
# exposes the addLevelName helper" — mamba returns False), `hasattr(
# bdb, 'checkfuncname')` (the documented "bdb exposes the
# checkfuncname helper" — mamba returns False), `hasattr(socket, '
# SOCK_RAW')` (the documented "socket exposes the SOCK_RAW socket-
# kind constant" — mamba returns False), `hasattr(socket, 'SHUT_RDWR'
# )` (the documented "socket exposes the SHUT_RDWR shutdown-direction
# constant" — mamba returns False), `socket.SHUT_RD == 0` (the
# documented "socket.SHUT_RD is the integer 0" — mamba returns None —
# constant unresolved), and `hasattr(socket, 'gaierror')` (the
# documented "socket exposes the gaierror exception" — mamba returns
# False).
# Ten-pack pinned to atomic 313.
#
# Behavioral edges that CONFORM on mamba (logging — hasattr getLogger
# /basicConfig/DEBUG/INFO/WARNING/ERROR/CRITICAL/debug/info/warning/
# error/critical + DEBUG==10/INFO==20/WARNING==30/ERROR==40/CRITICAL
# ==50 + getLogger('x').name == 'x'. warnings — hasattr warn/warn_
# explicit/filterwarnings/simplefilter/resetwarnings/showwarning/
# formatwarning/catch_warnings/WarningMessage/filters. bdb — hasattr
# Bdb/Breakpoint/BdbQuit/set_trace. wave — hasattr Wave_read/Wave_
# write/open/Error. socket — hasattr AF_INET/AF_INET6/AF_UNIX/SOCK_
# STREAM/SOCK_DGRAM/socket/gethostname + AF_INET == 2 + SOCK_STREAM
# == 1 + type(gethostname()) str) are covered in the matching pass
# fixture `test_logging_warnings_wave_socket_value_ops`.
import logging
import bdb
import socket


_ledger: list[int] = []

# 1) hasattr(logging, 'Logger') — Logger class
#    (mamba: returns False)
assert hasattr(logging, "Logger") == True; _ledger.append(1)

# 2) hasattr(logging, 'StreamHandler') — StreamHandler class
#    (mamba: returns False)
assert hasattr(logging, "StreamHandler") == True; _ledger.append(1)

# 3) hasattr(logging, 'NOTSET') — NOTSET level constant
#    (mamba: returns False)
assert hasattr(logging, "NOTSET") == True; _ledger.append(1)

# 4) type(logging.getLogger('x')).__name__ == 'Logger' — Logger instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(logging.getLogger("x")).__name__ == "Logger"; _ledger.append(1)

# 5) hasattr(logging, 'addLevelName') — addLevelName helper
#    (mamba: returns False)
assert hasattr(logging, "addLevelName") == True; _ledger.append(1)

# 6) hasattr(bdb, 'checkfuncname') — checkfuncname helper
#    (mamba: returns False)
assert hasattr(bdb, "checkfuncname") == True; _ledger.append(1)

# 7) hasattr(socket, 'SOCK_RAW') — SOCK_RAW socket-kind constant
#    (mamba: returns False)
assert hasattr(socket, "SOCK_RAW") == True; _ledger.append(1)

# 8) hasattr(socket, 'SHUT_RDWR') — SHUT_RDWR shutdown-direction constant
#    (mamba: returns False)
assert hasattr(socket, "SHUT_RDWR") == True; _ledger.append(1)

# 9) socket.SHUT_RD == 0 — read-shutdown constant
#    (mamba: returns None — constant unresolved)
assert socket.SHUT_RD == 0; _ledger.append(1)

# 10) hasattr(socket, 'gaierror') — gaierror exception
#     (mamba: returns False)
assert hasattr(socket, "gaierror") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_logging_bdb_socket_silent {sum(_ledger)} asserts")
