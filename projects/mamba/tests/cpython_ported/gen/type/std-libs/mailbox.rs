use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/mailbox/BabylMessage__add_label__label_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_BabylMessage__add_label__label_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "BabylMessage__add_label__label_as_str_wrong"
# subject = "mailbox.BabylMessage.add_label(label: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.BabylMessage.add_label(label: str); call it with the wrong type.

typeshed contract: label is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import BabylMessage
obj = object.__new__(BabylMessage)
try:
    obj.add_label(12345)  # label: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/BabylMessage__remove_label__label_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_BabylMessage__remove_label__label_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "BabylMessage__remove_label__label_as_str_wrong"
# subject = "mailbox.BabylMessage.remove_label(label: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.BabylMessage.remove_label(label: str); call it with the wrong type.

typeshed contract: label is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import BabylMessage
obj = object.__new__(BabylMessage)
try:
    obj.remove_label(12345)  # label: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/BabylMessage__set_labels__labels_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_BabylMessage__set_labels__labels_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "BabylMessage__set_labels__labels_as_Iterable_wrong"
# subject = "mailbox.BabylMessage.set_labels(labels: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.BabylMessage.set_labels(labels: Iterable); call it with the wrong type.

typeshed contract: labels is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import BabylMessage
obj = object.__new__(BabylMessage)
try:
    obj.set_labels(_W())  # labels: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/BabylMessage__set_visible__visible_as__MessageData_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_BabylMessage__set_visible__visible_as__MessageData_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "BabylMessage__set_visible__visible_as__MessageData_wrong"
# subject = "mailbox.BabylMessage.set_visible(visible: _MessageData)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.BabylMessage.set_visible(visible: _MessageData); call it with the wrong type.

typeshed contract: visible is _MessageData. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import BabylMessage
obj = object.__new__(BabylMessage)
try:
    obj.set_visible(_W())  # visible: _MessageData <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Babyl__get_bytes__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Babyl__get_bytes__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Babyl__get_bytes__key_as_str_wrong"
# subject = "mailbox.Babyl.get_bytes(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Babyl.get_bytes(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Babyl
obj = object.__new__(Babyl)
try:
    obj.get_bytes(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Babyl__get_file__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Babyl__get_file__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Babyl__get_file__key_as_str_wrong"
# subject = "mailbox.Babyl.get_file(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Babyl.get_file(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Babyl
obj = object.__new__(Babyl)
try:
    obj.get_file(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Babyl__get_message__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Babyl__get_message__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Babyl__get_message__key_as_str_wrong"
# subject = "mailbox.Babyl.get_message(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Babyl.get_message(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Babyl
obj = object.__new__(Babyl)
try:
    obj.get_message(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Babyl__init__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Babyl__init__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Babyl__init__path_as_StrPath_wrong"
# subject = "mailbox.Babyl.__init__(path: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Babyl.__init__(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Babyl
try:
    Babyl(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MHMessage__add_sequence__sequence_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MHMessage__add_sequence__sequence_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MHMessage__add_sequence__sequence_as_str_wrong"
# subject = "mailbox.MHMessage.add_sequence(sequence: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MHMessage.add_sequence(sequence: str); call it with the wrong type.

typeshed contract: sequence is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MHMessage
obj = object.__new__(MHMessage)
try:
    obj.add_sequence(12345)  # sequence: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MHMessage__remove_sequence__sequence_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MHMessage__remove_sequence__sequence_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MHMessage__remove_sequence__sequence_as_str_wrong"
# subject = "mailbox.MHMessage.remove_sequence(sequence: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MHMessage.remove_sequence(sequence: str); call it with the wrong type.

typeshed contract: sequence is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MHMessage
obj = object.__new__(MHMessage)
try:
    obj.remove_sequence(12345)  # sequence: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MHMessage__set_sequences__sequences_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MHMessage__set_sequences__sequences_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MHMessage__set_sequences__sequences_as_Iterable_wrong"
# subject = "mailbox.MHMessage.set_sequences(sequences: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MHMessage.set_sequences(sequences: Iterable); call it with the wrong type.

typeshed contract: sequences is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MHMessage
obj = object.__new__(MHMessage)
try:
    obj.set_sequences(_W())  # sequences: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH____contains____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH____contains____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH____contains____key_as_str_wrong"
# subject = "mailbox.MH.__contains__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.__contains__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MH
obj = object.__new__(MH)
try:
    obj.__contains__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH____setitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH____setitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH____setitem____key_as_str_wrong"
# subject = "mailbox.MH.__setitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.__setitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MH
obj = object.__new__(MH)
try:
    obj.__setitem__(12345, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__add__message_as__MessageData_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__add__message_as__MessageData_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__add__message_as__MessageData_wrong"
# subject = "mailbox.MH.add(message: _MessageData)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.add(message: _MessageData); call it with the wrong type.

typeshed contract: message is _MessageData. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
obj = object.__new__(MH)
try:
    obj.add(_W())  # message: _MessageData <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__add_folder__folder_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__add_folder__folder_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__add_folder__folder_as_StrPath_wrong"
# subject = "mailbox.MH.add_folder(folder: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.add_folder(folder: StrPath); call it with the wrong type.

typeshed contract: folder is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
obj = object.__new__(MH)
try:
    obj.add_folder(_W())  # folder: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__get_bytes__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__get_bytes__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__get_bytes__key_as_str_wrong"
# subject = "mailbox.MH.get_bytes(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.get_bytes(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MH
obj = object.__new__(MH)
try:
    obj.get_bytes(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__get_file__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__get_file__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__get_file__key_as_str_wrong"
# subject = "mailbox.MH.get_file(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.get_file(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MH
obj = object.__new__(MH)
try:
    obj.get_file(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__get_folder__folder_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__get_folder__folder_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__get_folder__folder_as_StrPath_wrong"
# subject = "mailbox.MH.get_folder(folder: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.get_folder(folder: StrPath); call it with the wrong type.

typeshed contract: folder is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
obj = object.__new__(MH)
try:
    obj.get_folder(_W())  # folder: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__get_message__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__get_message__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__get_message__key_as_str_wrong"
# subject = "mailbox.MH.get_message(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.get_message(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MH
obj = object.__new__(MH)
try:
    obj.get_message(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__init__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__init__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__init__path_as_StrPath_wrong"
# subject = "mailbox.MH.__init__(path: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.__init__(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
try:
    MH(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__remove__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__remove__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__remove__key_as_str_wrong"
# subject = "mailbox.MH.remove(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.remove(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MH
obj = object.__new__(MH)
try:
    obj.remove(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__remove_folder__folder_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__remove_folder__folder_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__remove_folder__folder_as_StrPath_wrong"
# subject = "mailbox.MH.remove_folder(folder: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.remove_folder(folder: StrPath); call it with the wrong type.

typeshed contract: folder is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
obj = object.__new__(MH)
try:
    obj.remove_folder(_W())  # folder: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MH__set_sequences__sequences_as_Mapping_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MH__set_sequences__sequences_as_Mapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__set_sequences__sequences_as_Mapping_wrong"
# subject = "mailbox.MH.set_sequences(sequences: Mapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.set_sequences(sequences: Mapping); call it with the wrong type.

typeshed contract: sequences is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
obj = object.__new__(MH)
try:
    obj.set_sequences(_W())  # sequences: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MMDF__init__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MMDF__init__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MMDF__init__path_as_StrPath_wrong"
# subject = "mailbox.MMDF.__init__(path: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MMDF.__init__(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MMDF
try:
    MMDF(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox____contains____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox____contains____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox____contains____key_as_str_wrong"
# subject = "mailbox.Mailbox.__contains__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.__contains__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.__contains__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox____delitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox____delitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox____delitem____key_as_str_wrong"
# subject = "mailbox.Mailbox.__delitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.__delitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.__delitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox____getitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox____getitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox____getitem____key_as_str_wrong"
# subject = "mailbox.Mailbox.__getitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.__getitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.__getitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox____setitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox____setitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox____setitem____key_as_str_wrong"
# subject = "mailbox.Mailbox.__setitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.__setitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.__setitem__(12345, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__add__message_as__MessageData_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__add__message_as__MessageData_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__add__message_as__MessageData_wrong"
# subject = "mailbox.Mailbox.add(message: _MessageData)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.add(message: _MessageData); call it with the wrong type.

typeshed contract: message is _MessageData. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.add(_W())  # message: _MessageData <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__discard__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__discard__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__discard__key_as_str_wrong"
# subject = "mailbox.Mailbox.discard(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.discard(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.discard(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__get__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__get__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__get__key_as_str_wrong"
# subject = "mailbox.Mailbox.get(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.get(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.get(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__get_bytes__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__get_bytes__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__get_bytes__key_as_str_wrong"
# subject = "mailbox.Mailbox.get_bytes(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.get_bytes(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.get_bytes(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__get_file__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__get_file__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__get_file__key_as_str_wrong"
# subject = "mailbox.Mailbox.get_file(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.get_file(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.get_file(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__get_message__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__get_message__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__get_message__key_as_str_wrong"
# subject = "mailbox.Mailbox.get_message(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.get_message(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.get_message(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__get_string__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__get_string__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__get_string__key_as_str_wrong"
# subject = "mailbox.Mailbox.get_string(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.get_string(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.get_string(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__init__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__init__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__init__path_as_StrPath_wrong"
# subject = "mailbox.Mailbox.__init__(path: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.__init__(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Mailbox
try:
    Mailbox(_W(), None)  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__pop__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__pop__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__pop__key_as_str_wrong"
# subject = "mailbox.Mailbox.pop(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.pop(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.pop(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__remove__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__remove__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__remove__key_as_str_wrong"
# subject = "mailbox.Mailbox.remove(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.remove(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.remove(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Mailbox__update__arg_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Mailbox__update__arg_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Mailbox__update__arg_as_typed_wrong"
# subject = "mailbox.Mailbox.update(arg: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Mailbox.update(arg: typed); call it with the wrong type.

typeshed contract: arg is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Mailbox
obj = object.__new__(Mailbox)
try:
    obj.update(_W())  # arg: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MaildirMessage__add_flag__flag_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MaildirMessage__add_flag__flag_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__add_flag__flag_as_str_wrong"
# subject = "mailbox.MaildirMessage.add_flag(flag: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.add_flag(flag: str); call it with the wrong type.

typeshed contract: flag is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.add_flag(12345)  # flag: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MaildirMessage__remove_flag__flag_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MaildirMessage__remove_flag__flag_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__remove_flag__flag_as_str_wrong"
# subject = "mailbox.MaildirMessage.remove_flag(flag: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.remove_flag(flag: str); call it with the wrong type.

typeshed contract: flag is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.remove_flag(12345)  # flag: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MaildirMessage__set_date__date_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MaildirMessage__set_date__date_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__set_date__date_as_float_wrong"
# subject = "mailbox.MaildirMessage.set_date(date: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.set_date(date: float); call it with the wrong type.

typeshed contract: date is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.set_date("not_a_float")  # date: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MaildirMessage__set_flags__flags_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MaildirMessage__set_flags__flags_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__set_flags__flags_as_Iterable_wrong"
# subject = "mailbox.MaildirMessage.set_flags(flags: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.set_flags(flags: Iterable); call it with the wrong type.

typeshed contract: flags is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.set_flags(_W())  # flags: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MaildirMessage__set_info__info_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MaildirMessage__set_info__info_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__set_info__info_as_str_wrong"
# subject = "mailbox.MaildirMessage.set_info(info: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.set_info(info: str); call it with the wrong type.

typeshed contract: info is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.set_info(12345)  # info: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/MaildirMessage__set_subdir__subdir_as_Literal_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_MaildirMessage__set_subdir__subdir_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MaildirMessage__set_subdir__subdir_as_Literal_wrong"
# subject = "mailbox.MaildirMessage.set_subdir(subdir: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MaildirMessage.set_subdir(subdir: Literal); call it with the wrong type.

typeshed contract: subdir is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MaildirMessage
obj = object.__new__(MaildirMessage)
try:
    obj.set_subdir(_W())  # subdir: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir____contains____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir____contains____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir____contains____key_as_str_wrong"
# subject = "mailbox.Maildir.__contains__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.__contains__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.__contains__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir____setitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir____setitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir____setitem____key_as_str_wrong"
# subject = "mailbox.Maildir.__setitem__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.__setitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.__setitem__(12345, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__add__message_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__add__message_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__add__message_as_typed_wrong"
# subject = "mailbox.Maildir.add(message: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.add(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.add(_W())  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__add_flag__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__add_flag__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__add_flag__key_as_str_wrong"
# subject = "mailbox.Maildir.add_flag(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.add_flag(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.add_flag(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__add_folder__folder_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__add_folder__folder_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__add_folder__folder_as_str_wrong"
# subject = "mailbox.Maildir.add_folder(folder: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.add_folder(folder: str); call it with the wrong type.

typeshed contract: folder is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.add_folder(12345)  # folder: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__get_bytes__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__get_bytes__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__get_bytes__key_as_str_wrong"
# subject = "mailbox.Maildir.get_bytes(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.get_bytes(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.get_bytes(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__get_file__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__get_file__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__get_file__key_as_str_wrong"
# subject = "mailbox.Maildir.get_file(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.get_file(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.get_file(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__get_flags__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__get_flags__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__get_flags__key_as_str_wrong"
# subject = "mailbox.Maildir.get_flags(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.get_flags(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.get_flags(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__get_folder__folder_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__get_folder__folder_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__get_folder__folder_as_str_wrong"
# subject = "mailbox.Maildir.get_folder(folder: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.get_folder(folder: str); call it with the wrong type.

typeshed contract: folder is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.get_folder(12345)  # folder: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__get_info__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__get_info__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__get_info__key_as_str_wrong"
# subject = "mailbox.Maildir.get_info(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.get_info(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.get_info(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__get_message__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__get_message__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__get_message__key_as_str_wrong"
# subject = "mailbox.Maildir.get_message(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.get_message(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.get_message(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__init__dirname_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__init__dirname_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__init__dirname_as_StrPath_wrong"
# subject = "mailbox.Maildir.__init__(dirname: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.__init__(dirname: StrPath); call it with the wrong type.

typeshed contract: dirname is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Maildir
try:
    Maildir(_W())  # dirname: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__remove__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__remove__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__remove__key_as_str_wrong"
# subject = "mailbox.Maildir.remove(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.remove(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.remove(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__remove_flag__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__remove_flag__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__remove_flag__key_as_str_wrong"
# subject = "mailbox.Maildir.remove_flag(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.remove_flag(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.remove_flag(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__remove_folder__folder_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__remove_folder__folder_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__remove_folder__folder_as_str_wrong"
# subject = "mailbox.Maildir.remove_folder(folder: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.remove_folder(folder: str); call it with the wrong type.

typeshed contract: folder is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.remove_folder(12345)  # folder: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__set_flags__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__set_flags__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__set_flags__key_as_str_wrong"
# subject = "mailbox.Maildir.set_flags(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.set_flags(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.set_flags(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Maildir__set_info__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Maildir__set_info__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Maildir__set_info__key_as_str_wrong"
# subject = "mailbox.Maildir.set_info(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Maildir.set_info(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mailbox import Maildir
obj = object.__new__(Maildir)
try:
    obj.set_info(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/Message__init__message_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_Message__init__message_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "Message__init__message_as_typed_wrong"
# subject = "mailbox.Message.__init__(message: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.Message.__init__(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import Message
try:
    Message(_W())  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/mailbox/mbox__init__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_mailbox_mbox__init__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "mbox__init__path_as_StrPath_wrong"
# subject = "mailbox.mbox.__init__(path: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.mbox.__init__(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import mbox
try:
    mbox(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
