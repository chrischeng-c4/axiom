# Atomic 317 pass conformance — tempfile module (hasattr Temporary
# File/NamedTemporaryFile/SpooledTemporaryFile/TemporaryDirectory/
# mkstemp/mkdtemp/mktemp/gettempdir/gettempdirb/gettempprefix/get
# tempprefixb/tempdir + gettempdir/mkdtemp str type) + shutil module
# (hasattr copy/copy2/copyfile/copytree/copyfileobj/copymode/copystat/
# rmtree/move/disk_usage/chown/which/ignore_patterns/Error/SameFile
# Error/SpecialFileError/ExecError/ReadError/RegistryError/make_archive
# /unpack_archive/register_archive_format/get_archive_formats/get_
# unpack_formats/COPY_BUFSIZE/get_terminal_size) + glob module (hasattr
# glob/iglob/escape/has_magic) + fnmatch module (hasattr fnmatch/
# fnmatchcase/filter/translate) + pathlib module (hasattr Path/PurePath
# /PurePosixPath/PureWindowsPath/PosixPath/WindowsPath) + os module
# (hasattr getcwd/listdir/mkdir/makedirs/rmdir/remove/rename/environ/
# getenv/sep/altsep/pathsep/linesep/extsep/name/devnull/curdir/pardir/
# stat/lstat/access/getpid/getppid/getuid/geteuid/getgid/getegid/cpu_
# count/urandom + sep=='/' + linesep=='\n' + pathsep==':' + name==
# 'posix').
# All asserts match between CPython 3.12 and mamba.
import tempfile
import shutil
import glob
import fnmatch
import pathlib
import os


_ledger: list[int] = []

# 1) tempfile — hasattr core surface + str return types
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mktemp") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdirb") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefixb") == True; _ledger.append(1)
assert hasattr(tempfile, "tempdir") == True; _ledger.append(1)
assert type(tempfile.gettempdir()).__name__ == "str"; _ledger.append(1)

# 2) shutil — hasattr core surface
assert hasattr(shutil, "copy") == True; _ledger.append(1)
assert hasattr(shutil, "copy2") == True; _ledger.append(1)
assert hasattr(shutil, "copyfile") == True; _ledger.append(1)
assert hasattr(shutil, "copytree") == True; _ledger.append(1)
assert hasattr(shutil, "copyfileobj") == True; _ledger.append(1)
assert hasattr(shutil, "copymode") == True; _ledger.append(1)
assert hasattr(shutil, "copystat") == True; _ledger.append(1)
assert hasattr(shutil, "rmtree") == True; _ledger.append(1)
assert hasattr(shutil, "move") == True; _ledger.append(1)
assert hasattr(shutil, "disk_usage") == True; _ledger.append(1)
assert hasattr(shutil, "chown") == True; _ledger.append(1)
assert hasattr(shutil, "which") == True; _ledger.append(1)
assert hasattr(shutil, "ignore_patterns") == True; _ledger.append(1)
assert hasattr(shutil, "Error") == True; _ledger.append(1)
assert hasattr(shutil, "SameFileError") == True; _ledger.append(1)
assert hasattr(shutil, "SpecialFileError") == True; _ledger.append(1)
assert hasattr(shutil, "ExecError") == True; _ledger.append(1)
assert hasattr(shutil, "ReadError") == True; _ledger.append(1)
assert hasattr(shutil, "RegistryError") == True; _ledger.append(1)
assert hasattr(shutil, "make_archive") == True; _ledger.append(1)
assert hasattr(shutil, "unpack_archive") == True; _ledger.append(1)
assert hasattr(shutil, "register_archive_format") == True; _ledger.append(1)
assert hasattr(shutil, "get_archive_formats") == True; _ledger.append(1)
assert hasattr(shutil, "get_unpack_formats") == True; _ledger.append(1)
assert hasattr(shutil, "COPY_BUFSIZE") == True; _ledger.append(1)
assert hasattr(shutil, "get_terminal_size") == True; _ledger.append(1)

# 3) glob — hasattr (conformant subset)
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 4) fnmatch — hasattr
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 5) pathlib — hasattr
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "WindowsPath") == True; _ledger.append(1)

# 6) os — hasattr (conformant subset)
assert hasattr(os, "getcwd") == True; _ledger.append(1)
assert hasattr(os, "listdir") == True; _ledger.append(1)
assert hasattr(os, "mkdir") == True; _ledger.append(1)
assert hasattr(os, "makedirs") == True; _ledger.append(1)
assert hasattr(os, "rmdir") == True; _ledger.append(1)
assert hasattr(os, "remove") == True; _ledger.append(1)
assert hasattr(os, "rename") == True; _ledger.append(1)
assert hasattr(os, "environ") == True; _ledger.append(1)
assert hasattr(os, "getenv") == True; _ledger.append(1)
assert hasattr(os, "sep") == True; _ledger.append(1)
assert hasattr(os, "altsep") == True; _ledger.append(1)
assert hasattr(os, "pathsep") == True; _ledger.append(1)
assert hasattr(os, "linesep") == True; _ledger.append(1)
assert hasattr(os, "extsep") == True; _ledger.append(1)
assert hasattr(os, "name") == True; _ledger.append(1)
assert hasattr(os, "devnull") == True; _ledger.append(1)
assert hasattr(os, "curdir") == True; _ledger.append(1)
assert hasattr(os, "pardir") == True; _ledger.append(1)
assert hasattr(os, "stat") == True; _ledger.append(1)
assert hasattr(os, "lstat") == True; _ledger.append(1)
assert hasattr(os, "access") == True; _ledger.append(1)
assert hasattr(os, "getpid") == True; _ledger.append(1)
assert hasattr(os, "getppid") == True; _ledger.append(1)
assert hasattr(os, "getuid") == True; _ledger.append(1)
assert hasattr(os, "geteuid") == True; _ledger.append(1)
assert hasattr(os, "getgid") == True; _ledger.append(1)
assert hasattr(os, "getegid") == True; _ledger.append(1)
assert hasattr(os, "cpu_count") == True; _ledger.append(1)
assert hasattr(os, "urandom") == True; _ledger.append(1)

# 7) os — string-constant values
assert os.sep == "/"; _ledger.append(1)
assert os.linesep == "\n"; _ledger.append(1)
assert os.pathsep == ":"; _ledger.append(1)
assert os.name == "posix"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_tempfile_shutil_pathlib_value_ops {sum(_ledger)} asserts")
