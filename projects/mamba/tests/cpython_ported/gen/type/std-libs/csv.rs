use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_csv/Dialect____new____dialect_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_Dialect____new____dialect_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "Dialect____new____dialect_as_typed_wrong"
# subject = "_csv.Dialect.__new__(dialect: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.Dialect.__new__(dialect: typed); call it with the wrong type.

typeshed contract: dialect is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import Dialect
obj = object.__new__(Dialect)
try:
    obj.__new__(_W())  # dialect: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/Writer__writerow__row_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_Writer__writerow__row_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "Writer__writerow__row_as_Iterable_wrong"
# subject = "_csv.Writer.writerow(row: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.Writer.writerow(row: Iterable); call it with the wrong type.

typeshed contract: row is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import Writer
obj = object.__new__(Writer)
try:
    obj.writerow(_W())  # row: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/Writer__writerows__rows_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_Writer__writerows__rows_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "Writer__writerows__rows_as_Iterable_wrong"
# subject = "_csv.Writer.writerows(rows: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.Writer.writerows(rows: Iterable); call it with the wrong type.

typeshed contract: rows is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import Writer
obj = object.__new__(Writer)
try:
    obj.writerows(_W())  # rows: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/field_size_limit__new_limit_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_field_size_limit__new_limit_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "field_size_limit__new_limit_as_int_wrong"
# subject = "_csv.field_size_limit(new_limit: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.field_size_limit(new_limit: int); call it with the wrong type.

typeshed contract: new_limit is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _csv import field_size_limit
try:
    field_size_limit("not_an_int")  # new_limit: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/get_dialect__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_get_dialect__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "get_dialect__name_as_str_wrong"
# subject = "_csv.get_dialect(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.get_dialect(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _csv import get_dialect
try:
    get_dialect(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/reader__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_reader__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "reader__iterable_as_Iterable_wrong"
# subject = "_csv.reader(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.reader(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import reader
try:
    reader(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/register_dialect__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_register_dialect__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "register_dialect__name_as_str_wrong"
# subject = "_csv.register_dialect(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.register_dialect(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _csv import register_dialect
try:
    register_dialect(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/unregister_dialect__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_unregister_dialect__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "unregister_dialect__name_as_str_wrong"
# subject = "_csv.unregister_dialect(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.unregister_dialect(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _csv import unregister_dialect
try:
    unregister_dialect(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_csv/writer__fileobj_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs__csv_writer__fileobj_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "writer__fileobj_as_SupportsWrite_wrong"
# subject = "_csv.writer(fileobj: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.writer(fileobj: SupportsWrite); call it with the wrong type.

typeshed contract: fileobj is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import writer
try:
    writer(_W())  # fileobj: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/csv/DictReader__init__f_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_csv_DictReader__init__f_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "DictReader__init__f_as_Iterable_wrong"
# subject = "csv.DictReader.__init__(f: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.DictReader.__init__(f: Iterable); call it with the wrong type.

typeshed contract: f is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from csv import DictReader
try:
    DictReader(_W(), None)  # f: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/csv/DictWriter__init__f_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_csv_DictWriter__init__f_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "DictWriter__init__f_as_SupportsWrite_wrong"
# subject = "csv.DictWriter.__init__(f: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.DictWriter.__init__(f: SupportsWrite); call it with the wrong type.

typeshed contract: f is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from csv import DictWriter
try:
    DictWriter(_W(), None)  # f: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/csv/DictWriter__writerow__rowdict_as_Mapping_wrong.py`.
#[test]
fn test_gen_type_std_libs_csv_DictWriter__writerow__rowdict_as_Mapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "DictWriter__writerow__rowdict_as_Mapping_wrong"
# subject = "csv.DictWriter.writerow(rowdict: Mapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.DictWriter.writerow(rowdict: Mapping); call it with the wrong type.

typeshed contract: rowdict is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from csv import DictWriter
obj = object.__new__(DictWriter)
try:
    obj.writerow(_W())  # rowdict: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/csv/DictWriter__writerows__rowdicts_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_csv_DictWriter__writerows__rowdicts_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "DictWriter__writerows__rowdicts_as_Iterable_wrong"
# subject = "csv.DictWriter.writerows(rowdicts: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.DictWriter.writerows(rowdicts: Iterable); call it with the wrong type.

typeshed contract: rowdicts is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from csv import DictWriter
obj = object.__new__(DictWriter)
try:
    obj.writerows(_W())  # rowdicts: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/csv/Sniffer__has_header__sample_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_csv_Sniffer__has_header__sample_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "Sniffer__has_header__sample_as_str_wrong"
# subject = "csv.Sniffer.has_header(sample: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.Sniffer.has_header(sample: str); call it with the wrong type.

typeshed contract: sample is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from csv import Sniffer
obj = object.__new__(Sniffer)
try:
    obj.has_header(12345)  # sample: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/csv/Sniffer__sniff__sample_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_csv_Sniffer__sniff__sample_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "type"
# case = "Sniffer__sniff__sample_as_str_wrong"
# subject = "csv.Sniffer.sniff(sample: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: csv.Sniffer.sniff(sample: str); call it with the wrong type.

typeshed contract: sample is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from csv import Sniffer
obj = object.__new__(Sniffer)
try:
    obj.sniff(12345)  # sample: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
