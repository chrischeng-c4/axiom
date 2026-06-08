# Atomic 300 pass conformance — signal module (hasattr SIGTERM/SIGINT/
# SIGHUP/SIGKILL/SIGABRT/SIGUSR1/signal/SIG_DFL/SIG_IGN/Signals) + sys
# module (hasattr version/version_info/platform/maxsize/executable/
# path/modules/argv/stdout/stderr/stdin/byteorder/float_info/int_info/
# hash_info/getrecursionlimit/exit + type sys.version='str' + type
# sys.maxsize='int' + sys.byteorder='little' + type sys.path='list' +
# type sys.modules='dict') + platform module (hasattr system/machine/
# python_version/release/processor/node/platform + python_version
# returns str) + getpass module (hasattr getuser/getpass + getuser
# returns str) + argparse module (hasattr ArgumentParser).
# All asserts match between CPython 3.12 and mamba.
import signal
import sys
import platform
import getpass
import argparse


_ledger: list[int] = []

# 1) signal — hasattr core surface
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGHUP") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGABRT") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR1") == True; _ledger.append(1)
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)

# 2) sys — hasattr core surface
assert hasattr(sys, "version") == True; _ledger.append(1)
assert hasattr(sys, "version_info") == True; _ledger.append(1)
assert hasattr(sys, "platform") == True; _ledger.append(1)
assert hasattr(sys, "maxsize") == True; _ledger.append(1)
assert hasattr(sys, "executable") == True; _ledger.append(1)
assert hasattr(sys, "path") == True; _ledger.append(1)
assert hasattr(sys, "modules") == True; _ledger.append(1)
assert hasattr(sys, "argv") == True; _ledger.append(1)
assert hasattr(sys, "stdout") == True; _ledger.append(1)
assert hasattr(sys, "stderr") == True; _ledger.append(1)
assert hasattr(sys, "stdin") == True; _ledger.append(1)
assert hasattr(sys, "byteorder") == True; _ledger.append(1)
assert hasattr(sys, "float_info") == True; _ledger.append(1)
assert hasattr(sys, "int_info") == True; _ledger.append(1)
assert hasattr(sys, "hash_info") == True; _ledger.append(1)
assert hasattr(sys, "getrecursionlimit") == True; _ledger.append(1)
assert hasattr(sys, "exit") == True; _ledger.append(1)

# 3) sys — value contracts
assert type(sys.version).__name__ == "str"; _ledger.append(1)
assert type(sys.maxsize).__name__ == "int"; _ledger.append(1)
assert sys.byteorder == "little"; _ledger.append(1)
assert type(sys.path).__name__ == "list"; _ledger.append(1)
assert type(sys.modules).__name__ == "dict"; _ledger.append(1)

# 4) platform — hasattr core surface (conformant subset)
assert hasattr(platform, "system") == True; _ledger.append(1)
assert hasattr(platform, "machine") == True; _ledger.append(1)
assert hasattr(platform, "python_version") == True; _ledger.append(1)
assert hasattr(platform, "release") == True; _ledger.append(1)
assert hasattr(platform, "processor") == True; _ledger.append(1)
assert hasattr(platform, "node") == True; _ledger.append(1)
assert hasattr(platform, "platform") == True; _ledger.append(1)

# 5) platform — value contracts (conformant subset)
assert isinstance(platform.python_version(), str) == True; _ledger.append(1)

# 6) getpass — hasattr core surface (conformant subset)
assert hasattr(getpass, "getuser") == True; _ledger.append(1)
assert hasattr(getpass, "getpass") == True; _ledger.append(1)

# 7) getpass — value contracts
assert isinstance(getpass.getuser(), str) == True; _ledger.append(1)

# 8) argparse — hasattr core surface (conformant subset)
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_signal_sys_platform_getpass_value_ops {sum(_ledger)} asserts")
