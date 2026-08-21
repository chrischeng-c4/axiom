# Operational AssertionPass seed for the value contract of the
# `sys` / `sysconfig` / `pathlib` / `subprocess` / `signal`
# five-pack pinned to atomic 214: `sys` (the documented
# partial module-level helper / sentinel identifier hasattr
# surface — `argv` / `path` / `modules` / `platform` /
# `version` / `version_info` / `maxsize` / `byteorder` /
# `executable` / `prefix` / `exec_prefix` / `base_prefix` /
# `base_exec_prefix` / `stdin` / `stdout` / `stderr` /
# `exit` / `getsizeof` / `getrecursionlimit` /
# `setrecursionlimit` / `intern` / `getrefcount` /
# `implementation` / `flags` / `float_info` / `int_info` /
# `hash_info` / `is_finalizing` + the documented
# `type(sys.platform).__name__ == "str"` /
# `type(sys.version).__name__ == "str"` /
# `type(sys.maxsize).__name__ == "int"` /
# `sys.maxsize > 0` /
# `sys.byteorder in ("little", "big")` /
# `len(sys.argv) >= 1` /
# `type(sys.argv).__name__ == "list"` /
# `type(sys.path).__name__ == "list"` /
# `type(sys.modules).__name__ == "dict"` /
# `sys.version_info.major == 3` /
# `sys.getrecursionlimit() > 0` interpreter-introspection
# value contract), `sysconfig` (the documented partial
# module-level helper identifier hasattr surface —
# `get_config_var` / `get_config_vars` / `get_paths` /
# `get_path` / `get_platform` / `get_python_version` /
# `get_path_names` / `get_scheme_names` /
# `get_default_scheme` + the documented
# `type(sysconfig.get_python_version()).__name__ == "str"`
# / `type(sysconfig.get_platform()).__name__ == "str"` /
# `type(sysconfig.get_paths()).__name__ == "dict"`
# sysconfig accessor value contract), `pathlib` (the
# documented full module-level class identifier hasattr
# surface — `Path` / `PurePath` / `PurePosixPath` /
# `PureWindowsPath` / `PosixPath` / `WindowsPath`),
# `subprocess` (the documented full module-level helper
# / class / exception / sentinel identifier hasattr
# surface — `run` / `call` / `check_call` /
# `check_output` / `Popen` / `PIPE` / `STDOUT` /
# `DEVNULL` / `CalledProcessError` / `TimeoutExpired` /
# `CompletedProcess` / `SubprocessError`), and `signal`
# (the documented full module-level helper / sentinel
# identifier hasattr surface — `signal` / `SIGINT` /
# `SIGTERM` / `SIGKILL` / `SIGHUP` / `SIG_DFL` /
# `SIG_IGN` / `Signals` / `Handlers` / `getsignal` /
# `default_int_handler` / `raise_signal` / `strsignal`
# + the documented `signal.SIGINT > 0` integer-sentinel
# value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(sys, "maxunicode") / "thread_info" all False on
# mamba + type(sys.version_info).__name__ == "version_info"
# collapses to "dict" on mamba + type(sys.implementation)
# .__name__ == "SimpleNamespace" collapses to "dict" on
# mamba, hasattr(sysconfig, "get_preferred_scheme") /
# "parse_config_h" all False on mamba,
# str(pathlib.Path("/tmp/foo/bar.txt")) ==
# "/tmp/foo/bar.txt" + Path .name / .stem / .suffix /
# .parent attribute access + Path.is_absolute() method
# all collapse on mamba, type(signal.SIGINT).__name__ ==
# "Signals" / type(signal.SIG_DFL).__name__ == "Handlers"
# both collapse to "int" on mamba +
# str(signal.strsignal(2)) startswith "I" collapses to
# False on mamba) are covered in the matching spec
# fixture `lang_sys_sysconfig_pathlib_signal_silent`.
import sys
import sysconfig
import pathlib
import subprocess
import signal


_ledger: list[int] = []

# 1) sys — partial module hasattr surface
#    (maxunicode / thread_info DIVERGE on mamba — moved to
#    spec)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "prefix") == True; _ledger.append(1)
assert hasattr(sys, "exec_prefix") == True; _ledger.append(1)
assert hasattr(sys, "base_prefix") == True; _ledger.append(1)
assert hasattr(sys, "base_exec_prefix") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)
assert hasattr(sys, "getsizeof") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "setrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "intern") == True; _ledger.append(1)
assert hasattr(sys, "getrefcount") == True; _ledger.append(1)
assert hasattr(sys, "implementation") == True; _ledger.append(1)
assert hasattr(sys, "flags") == True; _ledger.append(1)
assert hasattr(sys, "float_info") == True; _ledger.append(1)
assert hasattr(sys, "int_info") == True; _ledger.append(1)
assert hasattr(sys, "hash_info") == True; _ledger.append(1)
assert hasattr(sys, "is_finalizing") == True; _ledger.append(1)

# 2) sys — interpreter-introspection value contract
#    (type(sys.version_info).__name__ "version_info" and
#    type(sys.implementation).__name__ "SimpleNamespace" both
#    DIVERGE on mamba — moved to spec)
assert type(sys.platform).__name__ == "str"; _ledger.append(1)
assert type(sys.version).__name__ == "str"; _ledger.append(1)
assert type(sys.maxsize).__name__ == "int"; _ledger.append(1)
assert sys.maxsize > 0; _ledger.append(1)
assert sys.byteorder in ("little", "big"); _ledger.append(1)
assert len(sys.argv) >= 1; _ledger.append(1)
assert type(sys.argv).__name__ == "list"; _ledger.append(1)
assert type(sys.path).__name__ == "list"; _ledger.append(1)
assert type(sys.modules).__name__ == "dict"; _ledger.append(1)
assert sys.version_info.major == 3; _ledger.append(1)
assert sys.getrecursionlimit() > 0; _ledger.append(1)

# 3) sysconfig — partial module hasattr surface
#    (get_preferred_scheme / parse_config_h DIVERGE on mamba
#    — moved to spec)
assert hasattr(sysconfig, "get_config_var") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_config_vars") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_paths") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_path") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_platform") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_python_version") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_path_names") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_scheme_names") == True; _ledger.append(1)
assert hasattr(sysconfig, "get_default_scheme") == True; _ledger.append(1)

# 4) sysconfig — accessor value contract
assert type(sysconfig.get_python_version()).__name__ == "str"; _ledger.append(1)
assert type(sysconfig.get_platform()).__name__ == "str"; _ledger.append(1)
assert type(sysconfig.get_paths()).__name__ == "dict"; _ledger.append(1)

# 5) pathlib — full module hasattr surface
#    (Path() value contracts DIVERGE on mamba — moved to
#    spec)
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "WindowsPath") == True; _ledger.append(1)

# 6) subprocess — full module hasattr surface
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)
assert hasattr(subprocess, "SubprocessError") == True; _ledger.append(1)

# 7) signal — full module hasattr surface
#    (type(signal.SIGINT).__name__ "Signals" /
#    type(signal.SIG_DFL).__name__ "Handlers" type-identity
#    + strsignal collapse DIVERGE on mamba — moved to spec)
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGHUP") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)
assert hasattr(signal, "Handlers") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "default_int_handler") == True; _ledger.append(1)
assert hasattr(signal, "raise_signal") == True; _ledger.append(1)
assert hasattr(signal, "strsignal") == True; _ledger.append(1)

# 8) signal — integer-sentinel value contract
assert signal.SIGINT > 0; _ledger.append(1)
assert signal.SIGTERM > 0; _ledger.append(1)

# NB: hasattr(sys, "maxunicode") / "thread_info" all False
# on mamba + type(sys.version_info).__name__ ==
# "version_info" collapses to "dict" on mamba +
# type(sys.implementation).__name__ == "SimpleNamespace"
# collapses to "dict" on mamba, hasattr(sysconfig,
# "get_preferred_scheme") / "parse_config_h" all False
# on mamba, str(pathlib.Path("/tmp/foo/bar.txt")) ==
# "/tmp/foo/bar.txt" + Path .name / .stem / .suffix /
# .parent attribute access + Path.is_absolute() method
# all collapse on mamba, type(signal.SIGINT).__name__ ==
# "Signals" / type(signal.SIG_DFL).__name__ == "Handlers"
# both collapse to "int" on mamba +
# str(signal.strsignal(2)) startswith "I" collapses to
# False on mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_sys_sysconfig_pathlib_subprocess_signal_value_ops {sum(_ledger)} asserts")
