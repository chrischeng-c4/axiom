# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: shutil — copy / copyfile / copytree / move / rmtree / which.
# Binary-mode open() is unsupported on mamba today (`open(p, "wb")` raises
# ValueError), so this seed exercises text-mode files only.
# shutil.disk_usage exists but returns a stub with zeroed total/free values —
# intentionally NOT exercised; tracked separately.
import shutil
import os
import tempfile

_ledger: list[int] = []

# Set up a workspace with one source file
_d = tempfile.mkdtemp()
_src = os.path.join(_d, "src.txt")
_fw = open(_src, "w")
_fw.write("hello\n")
_fw.close()

# shutil.copy returns the destination path and creates the file
_dst = os.path.join(_d, "dst.txt")
_r = shutil.copy(_src, _dst)
assert _r == _dst, "shutil.copy returns the destination path"
_ledger.append(1)

assert os.path.exists(_dst), "shutil.copy creates the destination file"
_ledger.append(1)

# shutil.copyfile also copies and returns the destination path
_dst2 = os.path.join(_d, "dst2.txt")
_r2 = shutil.copyfile(_src, _dst2)
assert _r2 == _dst2, "shutil.copyfile returns the destination path"
_ledger.append(1)

assert os.path.exists(_dst2), "shutil.copyfile creates the destination file"
_ledger.append(1)

# shutil.copytree recursively copies a directory tree
_subsrc = os.path.join(_d, "subsrc")
os.makedirs(_subsrc)
_fw2 = open(os.path.join(_subsrc, "a.txt"), "w")
_fw2.write("a")
_fw2.close()
_subdst = os.path.join(_d, "subdst")
shutil.copytree(_subsrc, _subdst)
assert os.path.exists(_subdst), "shutil.copytree creates the destination directory"
_ledger.append(1)

assert os.path.exists(os.path.join(_subdst, "a.txt")), (
    "shutil.copytree copies inner files"
)
_ledger.append(1)

# shutil.move relocates the file: source disappears, destination appears
_moved = os.path.join(_d, "moved.txt")
shutil.move(_dst2, _moved)
assert not os.path.exists(_dst2), "shutil.move removes the source file"
_ledger.append(1)

assert os.path.exists(_moved), "shutil.move creates the destination file"
_ledger.append(1)

# shutil.which finds a well-known POSIX binary
_ls = shutil.which("ls")
assert isinstance(_ls, str) and len(_ls) > 0 and "ls" in _ls, (
    "shutil.which('ls') returns a non-empty path containing 'ls'"
)
_ledger.append(1)

# shutil.which returns None for a name that cannot be on PATH
assert shutil.which("nonexistent_program_xyz_12345_mamba") is None, (
    "shutil.which on a missing program returns None"
)
_ledger.append(1)

# shutil.rmtree removes the entire workspace
shutil.rmtree(_d)
assert not os.path.exists(_d), "shutil.rmtree removes the workspace directory"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_shutil {sum(_ledger)} asserts")
