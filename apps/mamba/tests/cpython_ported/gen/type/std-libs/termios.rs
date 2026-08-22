use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/termios/tcdrain__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcdrain__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcdrain__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcdrain(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcdrain(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcdrain
try:
    tcdrain(_W())  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcflow__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcflow__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcflow__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcflow(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcflow(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcflow
try:
    tcflow(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcflush__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcflush__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcflush__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcflush(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcflush(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcflush
try:
    tcflush(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcgetattr__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcgetattr__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcgetattr__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcgetattr(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcgetattr(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcgetattr
try:
    tcgetattr(_W())  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcgetwinsize__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcgetwinsize__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcgetwinsize__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcgetwinsize(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcgetwinsize(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcgetwinsize
try:
    tcgetwinsize(_W())  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcsendbreak__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcsendbreak__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcsendbreak__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcsendbreak(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcsendbreak(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcsendbreak
try:
    tcsendbreak(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcsetattr__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcsetattr__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcsetattr__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcsetattr(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcsetattr(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcsetattr
try:
    tcsetattr(_W(), 0, None)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/termios/tcsetwinsize__fd_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_termios_tcsetwinsize__fd_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcsetwinsize__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcsetwinsize(fd: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcsetwinsize(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcsetwinsize
try:
    tcsetwinsize(_W(), None)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
