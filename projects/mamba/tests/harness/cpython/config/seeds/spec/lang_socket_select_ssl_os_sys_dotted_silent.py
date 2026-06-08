# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of `socket.getservbyname /
# socket.error / socket.timeout / socket.gaierror / socket.herror /
# socket.SO_REUSEADDR / socket.SOL_SOCKET / socket.IPPROTO_TCP /
# socket.IPPROTO_UDP` (the documented top-level surface — mamba
# does not expose them), `type(socket.AF_INET).__name__ /
# type(socket.SOCK_STREAM).__name__` (the documented
# "AddressFamily / SocketKind enum" type contract — mamba binds
# these to bare ints, so the type-name is silently `int`),
# `select.select / select.poll / select.kqueue / select.error`
# (the documented top-level surface — mamba does not expose any
# of them), `hasattr(ssl, "wrap_socket") == False` (the
# documented "wrap_socket removed in 3.12" surface — mamba
# silently keeps the legacy `ssl.wrap_socket` exposed),
# `"PATH" in os.environ` (the documented "PATH env var is
# always present" membership contract — mamba returns False),
# `sys.version_info[0]` (the documented "version_info is a
# named-tuple indexable as `[0]` for the major component" —
# mamba raises `KeyError '0'` at the call site because it binds
# version_info to a dict-shaped surrogate), and the dotted-import
# quirk where `urllib.request` resolves to `None` after the
# canonical `import urllib.request` (the documented "import X.Y
# binds X.Y submodule on X" — mamba silently leaves the
# submodule unbound, so `hasattr(urllib.request, "urlopen")`
# returns False unless aliased via `import urllib.request as _`).
# Ten-pack pinned to atomic 245.
#
# Behavioral edges that CONFORM on mamba (str.format positional/
# kwarg/pad/right/center/zero/hex/float/e specifiers +
# str.format_map + format() builtin + f-string basic/format/
# float; socket basic surface socket/AF_INET/AF_INET6/
# SOCK_STREAM/SOCK_DGRAM/AF_UNIX/gethostname/gethostbyname/
# getaddrinfo; ssl class surface SSLContext/
# create_default_context/CERT_NONE/CERT_REQUIRED/PROTOCOL_TLS/
# SSLError/Purpose; aliased urllib.request urlopen/Request/
# build_opener/install_opener/HTTPHandler + urllib.error
# URLError/HTTPError/ContentTooShortError; os.environ hasattr +
# HOME type; sys flags/version_info/platform/maxsize/executable/
# modules/argv/stdout/stderr/stdin/path hasattr + maxsize int +
# platform str types; time monotonic/perf_counter/process_time/
# time/sleep/time_ns/monotonic_ns hasattr + monotonic/time
# positive) are covered in the matching pass fixture
# `test_string_fmt_socket_ssl_urllib_sys_time_value_ops`.
from typing import Any
import socket as _socket_mod
import select as _select_mod
import ssl as _ssl_mod
import os as _os_mod
import sys as _sys_mod
import urllib.request  # noqa: F401  (deliberately not aliased — exercises dotted-import quirk)

socket_mod: Any = _socket_mod
select_mod: Any = _select_mod
ssl_mod: Any = _ssl_mod
os_mod: Any = _os_mod
sys_mod: Any = _sys_mod
urllib_mod: Any = __import__("urllib")


_ledger: list[int] = []

# 1) socket deep surface — getservbyname
#    (mamba: missing)
assert hasattr(socket_mod, "getservbyname") == True; _ledger.append(1)

# 2) socket.error / socket.timeout / socket.gaierror / socket.herror
#    (mamba: all missing)
assert hasattr(socket_mod, "error") == True; _ledger.append(1)
assert hasattr(socket_mod, "timeout") == True; _ledger.append(1)
assert hasattr(socket_mod, "gaierror") == True; _ledger.append(1)
assert hasattr(socket_mod, "herror") == True; _ledger.append(1)

# 3) socket constants — SO_REUSEADDR / SOL_SOCKET / IPPROTO_TCP / IPPROTO_UDP
#    (mamba: all missing)
assert hasattr(socket_mod, "SO_REUSEADDR") == True; _ledger.append(1)
assert hasattr(socket_mod, "SOL_SOCKET") == True; _ledger.append(1)
assert hasattr(socket_mod, "IPPROTO_TCP") == True; _ledger.append(1)
assert hasattr(socket_mod, "IPPROTO_UDP") == True; _ledger.append(1)

# 4) socket.AF_INET / SOCK_STREAM enum type contract
#    (mamba: returns bare int, type name is 'int')
assert type(socket_mod.AF_INET).__name__ == "AddressFamily"; _ledger.append(1)
assert type(socket_mod.SOCK_STREAM).__name__ == "SocketKind"; _ledger.append(1)

# 5) select top-level surface
#    (mamba: entire module surface missing)
assert hasattr(select_mod, "select") == True; _ledger.append(1)
assert hasattr(select_mod, "poll") == True; _ledger.append(1)
assert hasattr(select_mod, "kqueue") == True; _ledger.append(1)
assert hasattr(select_mod, "error") == True; _ledger.append(1)

# 6) ssl.wrap_socket — Python 3.12 removed it
#    (mamba: silently keeps the legacy ssl.wrap_socket exposed)
assert hasattr(ssl_mod, "wrap_socket") == False; _ledger.append(1)

# 7) os.environ PATH membership
#    (mamba: returns False)
assert ("PATH" in os_mod.environ) == True; _ledger.append(1)

# 8) sys.version_info[0] — named-tuple major-component indexing
#    (mamba: raises KeyError '0' — version_info bound to dict surrogate)
assert sys_mod.version_info[0] == 3; _ledger.append(1)

# 9) dotted-import quirk — urllib.request submodule binding
#    (mamba: urllib.request resolves to None after `import urllib.request`)
assert hasattr(urllib_mod.request, "urlopen") == True; _ledger.append(1)

# 10) dotted-import quirk — urllib.request truthiness
#     (mamba: urllib.request is None after canonical dotted import)
assert (urllib_mod.request is None) == False; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_socket_select_ssl_os_sys_dotted_silent {sum(_ledger)} asserts")
