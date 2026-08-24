use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/fcntl/fcntl__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_fcntl_fcntl__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "type"
# case = "fcntl__fd_as_FileDescriptorLike_wrong"
# subject = "fcntl.fcntl(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fcntl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fcntl.fcntl(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fcntl import fcntl
try:
    fcntl(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fcntl/flock__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_fcntl_flock__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "type"
# case = "flock__fd_as_FileDescriptorLike_wrong"
# subject = "fcntl.flock(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fcntl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fcntl.flock(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fcntl import flock
try:
    flock(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fcntl/ioctl__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_fcntl_ioctl__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "type"
# case = "ioctl__fd_as_FileDescriptorLike_wrong"
# subject = "fcntl.ioctl(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fcntl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fcntl.ioctl(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fcntl import ioctl
try:
    ioctl(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fcntl/lockf__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_fcntl_lockf__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "type"
# case = "lockf__fd_as_FileDescriptorLike_wrong"
# subject = "fcntl.lockf(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fcntl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fcntl.lockf(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fcntl import lockf
try:
    lockf(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
