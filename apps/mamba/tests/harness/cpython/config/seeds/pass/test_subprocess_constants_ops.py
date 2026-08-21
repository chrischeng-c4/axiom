# Operational AssertionPass seed for `subprocess` stream-redirection
# sentinel constants.
# Surface: PIPE = -1, STDOUT = -2, DEVNULL = -3. These sentinel values
# are part of the documented subprocess API used to direct a child
# process's stdin/stdout/stderr.
# Companion to stub/test_subprocess.py — vendored unittest seed.
import subprocess
_ledger: list[int] = []
assert subprocess.PIPE == -1; _ledger.append(1)
assert subprocess.STDOUT == -2; _ledger.append(1)
assert subprocess.DEVNULL == -3; _ledger.append(1)
# All three sentinels are distinct
assert subprocess.PIPE != subprocess.STDOUT; _ledger.append(1)
assert subprocess.STDOUT != subprocess.DEVNULL; _ledger.append(1)
assert subprocess.PIPE != subprocess.DEVNULL; _ledger.append(1)
# All three are negative (so they cannot collide with a real file
# descriptor)
assert subprocess.PIPE < 0; _ledger.append(1)
assert subprocess.STDOUT < 0; _ledger.append(1)
assert subprocess.DEVNULL < 0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_subprocess_constants_ops {sum(_ledger)} asserts")
