use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/logging_config/BaseConfigurator__as_tuple__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_BaseConfigurator__as_tuple__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "BaseConfigurator__as_tuple__value_as_typed_wrong"
# subject = "logging.config.BaseConfigurator.as_tuple(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.BaseConfigurator.as_tuple(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import BaseConfigurator
obj = object.__new__(BaseConfigurator)
try:
    obj.as_tuple(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/BaseConfigurator__cfg_convert__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_BaseConfigurator__cfg_convert__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "BaseConfigurator__cfg_convert__value_as_str_wrong"
# subject = "logging.config.BaseConfigurator.cfg_convert(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.BaseConfigurator.cfg_convert(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import BaseConfigurator
obj = object.__new__(BaseConfigurator)
try:
    obj.cfg_convert(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/BaseConfigurator__configure_custom__config_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_BaseConfigurator__configure_custom__config_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "BaseConfigurator__configure_custom__config_as_dict_wrong"
# subject = "logging.config.BaseConfigurator.configure_custom(config: dict)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.BaseConfigurator.configure_custom(config: dict); call it with the wrong type.

typeshed contract: config is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import BaseConfigurator
obj = object.__new__(BaseConfigurator)
try:
    obj.configure_custom(12345)  # config: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/BaseConfigurator__ext_convert__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_BaseConfigurator__ext_convert__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "BaseConfigurator__ext_convert__value_as_str_wrong"
# subject = "logging.config.BaseConfigurator.ext_convert(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.BaseConfigurator.ext_convert(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import BaseConfigurator
obj = object.__new__(BaseConfigurator)
try:
    obj.ext_convert(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/BaseConfigurator__init__config_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_BaseConfigurator__init__config_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "BaseConfigurator__init__config_as_typed_wrong"
# subject = "logging.config.BaseConfigurator.__init__(config: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.BaseConfigurator.__init__(config: typed); call it with the wrong type.

typeshed contract: config is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import BaseConfigurator
try:
    BaseConfigurator(_W())  # config: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/BaseConfigurator__resolve__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_BaseConfigurator__resolve__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "BaseConfigurator__resolve__s_as_str_wrong"
# subject = "logging.config.BaseConfigurator.resolve(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.BaseConfigurator.resolve(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import BaseConfigurator
obj = object.__new__(BaseConfigurator)
try:
    obj.resolve(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingDict____getitem____key_as_Hashable_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingDict____getitem____key_as_Hashable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingDict____getitem____key_as_Hashable_wrong"
# subject = "logging.config.ConvertingDict.__getitem__(key: Hashable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingDict.__getitem__(key: Hashable); call it with the wrong type.

typeshed contract: key is Hashable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingDict
obj = object.__new__(ConvertingDict)
try:
    obj.__getitem__(_W())  # key: Hashable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingDict__get__key_as_Hashable_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingDict__get__key_as_Hashable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingDict__get__key_as_Hashable_wrong"
# subject = "logging.config.ConvertingDict.get(key: Hashable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingDict.get(key: Hashable); call it with the wrong type.

typeshed contract: key is Hashable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingDict
obj = object.__new__(ConvertingDict)
try:
    obj.get(_W())  # key: Hashable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingDict__pop__key_as_Hashable_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingDict__pop__key_as_Hashable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingDict__pop__key_as_Hashable_wrong"
# subject = "logging.config.ConvertingDict.pop(key: Hashable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingDict.pop(key: Hashable); call it with the wrong type.

typeshed contract: key is Hashable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingDict
obj = object.__new__(ConvertingDict)
try:
    obj.pop(_W())  # key: Hashable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingList____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingList____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingList____getitem____key_as_SupportsIndex_wrong"
# subject = "logging.config.ConvertingList.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingList.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingList
obj = object.__new__(ConvertingList)
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingList____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingList____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingList____getitem____key_as_slice_wrong"
# subject = "logging.config.ConvertingList.__getitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingList.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingList
obj = object.__new__(ConvertingList)
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingList__pop__idx_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingList__pop__idx_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingList__pop__idx_as_SupportsIndex_wrong"
# subject = "logging.config.ConvertingList.pop(idx: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingList.pop(idx: SupportsIndex); call it with the wrong type.

typeshed contract: idx is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingList
obj = object.__new__(ConvertingList)
try:
    obj.pop(_W())  # idx: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingMixin__convert_with_key__replace_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingMixin__convert_with_key__replace_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingMixin__convert_with_key__replace_as_bool_wrong"
# subject = "logging.config.ConvertingMixin.convert_with_key(replace: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingMixin.convert_with_key(replace: bool); call it with the wrong type.

typeshed contract: replace is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import ConvertingMixin
obj = object.__new__(ConvertingMixin)
try:
    obj.convert_with_key(None, None, "not_a_bool")  # replace: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingTuple____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingTuple____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingTuple____getitem____key_as_SupportsIndex_wrong"
# subject = "logging.config.ConvertingTuple.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingTuple.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingTuple
obj = object.__new__(ConvertingTuple)
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/ConvertingTuple____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_ConvertingTuple____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "ConvertingTuple____getitem____key_as_slice_wrong"
# subject = "logging.config.ConvertingTuple.__getitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.ConvertingTuple.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import ConvertingTuple
obj = object.__new__(ConvertingTuple)
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__add_filters__filterer_as_Filterer_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__add_filters__filterer_as_Filterer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__add_filters__filterer_as_Filterer_wrong"
# subject = "logging.config.DictConfigurator.add_filters(filterer: Filterer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.add_filters(filterer: Filterer); call it with the wrong type.

typeshed contract: filterer is Filterer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.add_filters(_W(), None)  # filterer: Filterer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__add_handlers__logger_as_Logger_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__add_handlers__logger_as_Logger_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__add_handlers__logger_as_Logger_wrong"
# subject = "logging.config.DictConfigurator.add_handlers(logger: Logger)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.add_handlers(logger: Logger); call it with the wrong type.

typeshed contract: logger is Logger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.add_handlers(_W(), None)  # logger: Logger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__common_logger_config__logger_as_Logger_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__common_logger_config__logger_as_Logger_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__common_logger_config__logger_as_Logger_wrong"
# subject = "logging.config.DictConfigurator.common_logger_config(logger: Logger)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.common_logger_config(logger: Logger); call it with the wrong type.

typeshed contract: logger is Logger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.common_logger_config(_W(), None)  # logger: Logger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__configure_filter__config_as__FilterConfiguration_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__configure_filter__config_as__FilterConfiguration_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_filter__config_as__FilterConfiguration_wrong"
# subject = "logging.config.DictConfigurator.configure_filter(config: _FilterConfiguration)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_filter(config: _FilterConfiguration); call it with the wrong type.

typeshed contract: config is _FilterConfiguration. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_filter(_W())  # config: _FilterConfiguration <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__configure_formatter__config_as__FormatterConfiguration_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__configure_formatter__config_as__FormatterConfiguration_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_formatter__config_as__FormatterConfiguration_wrong"
# subject = "logging.config.DictConfigurator.configure_formatter(config: _FormatterConfiguration)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_formatter(config: _FormatterConfiguration); call it with the wrong type.

typeshed contract: config is _FormatterConfiguration. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_formatter(_W())  # config: _FormatterConfiguration <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__configure_handler__config_as__HandlerConfiguration_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__configure_handler__config_as__HandlerConfiguration_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_handler__config_as__HandlerConfiguration_wrong"
# subject = "logging.config.DictConfigurator.configure_handler(config: _HandlerConfiguration)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_handler(config: _HandlerConfiguration); call it with the wrong type.

typeshed contract: config is _HandlerConfiguration. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_handler(_W())  # config: _HandlerConfiguration <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__configure_logger__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__configure_logger__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_logger__name_as_str_wrong"
# subject = "logging.config.DictConfigurator.configure_logger(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_logger(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_logger(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/DictConfigurator__configure_root__config_as__LoggerConfiguration_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_DictConfigurator__configure_root__config_as__LoggerConfiguration_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__configure_root__config_as__LoggerConfiguration_wrong"
# subject = "logging.config.DictConfigurator.configure_root(config: _LoggerConfiguration)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.configure_root(config: _LoggerConfiguration); call it with the wrong type.

typeshed contract: config is _LoggerConfiguration. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.configure_root(_W())  # config: _LoggerConfiguration <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/dictConfig__config_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_dictConfig__config_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "dictConfig__config_as_typed_wrong"
# subject = "logging.config.dictConfig(config: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.dictConfig(config: typed); call it with the wrong type.

typeshed contract: config is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import dictConfig
try:
    dictConfig(_W())  # config: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/fileConfig__fname_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_fileConfig__fname_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "fileConfig__fname_as_typed_wrong"
# subject = "logging.config.fileConfig(fname: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.fileConfig(fname: typed); call it with the wrong type.

typeshed contract: fname is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import fileConfig
try:
    fileConfig(_W())  # fname: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/listen__port_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_listen__port_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "listen__port_as_int_wrong"
# subject = "logging.config.listen(port: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.listen(port: int); call it with the wrong type.

typeshed contract: port is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import listen
try:
    listen("not_an_int")  # port: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/logging_config/valid_ident__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_logging_config_valid_ident__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "valid_ident__s_as_str_wrong"
# subject = "logging.config.valid_ident(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.valid_ident(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.config import valid_ident
try:
    valid_ident(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
