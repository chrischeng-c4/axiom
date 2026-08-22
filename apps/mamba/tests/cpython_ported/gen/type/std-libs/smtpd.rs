use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/smtpd/MailmanProxy__process_message__peer_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_MailmanProxy__process_message__peer_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "MailmanProxy__process_message__peer_as__Address_wrong"
# subject = "smtpd.MailmanProxy.process_message(peer: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.MailmanProxy.process_message(peer: _Address); call it with the wrong type.

typeshed contract: peer is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import MailmanProxy
obj = object.__new__(MailmanProxy)
try:
    obj.process_message(_W(), "", None, None)  # peer: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/PureProxy__process_message__peer_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_PureProxy__process_message__peer_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "PureProxy__process_message__peer_as__Address_wrong"
# subject = "smtpd.PureProxy.process_message(peer: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.PureProxy.process_message(peer: _Address); call it with the wrong type.

typeshed contract: peer is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import PureProxy
obj = object.__new__(PureProxy)
try:
    obj.process_message(_W(), "", None, None)  # peer: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__collect_incoming_data__data_as_bytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__collect_incoming_data__data_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__collect_incoming_data__data_as_bytes_wrong"
# subject = "smtpd.SMTPChannel.collect_incoming_data(data: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.collect_incoming_data(data: bytes); call it with the wrong type.

typeshed contract: data is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.collect_incoming_data(12345)  # data: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__init__server_as_SMTPServer_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__init__server_as_SMTPServer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__init__server_as_SMTPServer_wrong"
# subject = "smtpd.SMTPChannel.__init__(server: SMTPServer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.__init__(server: SMTPServer); call it with the wrong type.

typeshed contract: server is SMTPServer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import SMTPChannel
try:
    SMTPChannel(_W(), None, None)  # server: SMTPServer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__push__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__push__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__push__msg_as_str_wrong"
# subject = "smtpd.SMTPChannel.push(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.push(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.push(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_DATA__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_DATA__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_DATA__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_DATA(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_DATA(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_DATA(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_EHLO__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_EHLO__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_EHLO__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_EHLO(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_EHLO(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_EHLO(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_EXPN__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_EXPN__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_EXPN__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_EXPN(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_EXPN(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_EXPN(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_HELO__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_HELO__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_HELO__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_HELO(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_HELO(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_HELO(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_HELP__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_HELP__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_HELP__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_HELP(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_HELP(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_HELP(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_MAIL__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_MAIL__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_MAIL__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_MAIL(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_MAIL(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_MAIL(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_NOOP__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_NOOP__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_NOOP__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_NOOP(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_NOOP(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_NOOP(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_QUIT__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_QUIT__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_QUIT__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_QUIT(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_QUIT(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_QUIT(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_RCPT__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_RCPT__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_RCPT__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_RCPT(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_RCPT(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_RCPT(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_RSET__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_RSET__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_RSET__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_RSET(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_RSET(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_RSET(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPChannel__smtp_VRFY__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPChannel__smtp_VRFY__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPChannel__smtp_VRFY__arg_as_str_wrong"
# subject = "smtpd.SMTPChannel.smtp_VRFY(arg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPChannel.smtp_VRFY(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from smtpd import SMTPChannel
obj = object.__new__(SMTPChannel)
try:
    obj.smtp_VRFY(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPServer__handle_accepted__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPServer__handle_accepted__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPServer__handle_accepted__conn_as_socket_wrong"
# subject = "smtpd.SMTPServer.handle_accepted(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPServer.handle_accepted(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import SMTPServer
obj = object.__new__(SMTPServer)
try:
    obj.handle_accepted(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPServer__init__localaddr_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPServer__init__localaddr_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPServer__init__localaddr_as__Address_wrong"
# subject = "smtpd.SMTPServer.__init__(localaddr: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPServer.__init__(localaddr: _Address); call it with the wrong type.

typeshed contract: localaddr is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import SMTPServer
try:
    SMTPServer(_W(), None)  # localaddr: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/smtpd/SMTPServer__process_message__peer_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_smtpd_SMTPServer__process_message__peer_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPServer__process_message__peer_as__Address_wrong"
# subject = "smtpd.SMTPServer.process_message(peer: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPServer.process_message(peer: _Address); call it with the wrong type.

typeshed contract: peer is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import SMTPServer
obj = object.__new__(SMTPServer)
try:
    obj.process_message(_W(), "", None, None)  # peer: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
