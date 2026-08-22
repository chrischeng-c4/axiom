use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/DOMEventStream____getitem____pos_as_Unused_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_DOMEventStream____getitem____pos_as_Unused_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "DOMEventStream____getitem____pos_as_Unused_wrong"
# subject = "xml.dom.pulldom.DOMEventStream.__getitem__(pos: Unused)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.DOMEventStream.__getitem__(pos: Unused); call it with the wrong type.

typeshed contract: pos is Unused. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import DOMEventStream
obj = object.__new__(DOMEventStream)
try:
    obj.__getitem__(_W())  # pos: Unused <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/DOMEventStream__expandNode__node_as_Document_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_DOMEventStream__expandNode__node_as_Document_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "DOMEventStream__expandNode__node_as_Document_wrong"
# subject = "xml.dom.pulldom.DOMEventStream.expandNode(node: Document)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.DOMEventStream.expandNode(node: Document); call it with the wrong type.

typeshed contract: node is Document. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import DOMEventStream
obj = object.__new__(DOMEventStream)
try:
    obj.expandNode(_W())  # node: Document <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/DOMEventStream__init__stream_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_DOMEventStream__init__stream_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "DOMEventStream__init__stream_as_typed_wrong"
# subject = "xml.dom.pulldom.DOMEventStream.__init__(stream: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.DOMEventStream.__init__(stream: typed); call it with the wrong type.

typeshed contract: stream is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import DOMEventStream
try:
    DOMEventStream(_W(), None, 0)  # stream: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/ErrorHandler__error__exception_as_BaseException_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_ErrorHandler__error__exception_as_BaseException_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "ErrorHandler__error__exception_as_BaseException_wrong"
# subject = "xml.dom.pulldom.ErrorHandler.error(exception: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.ErrorHandler.error(exception: BaseException); call it with the wrong type.

typeshed contract: exception is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import ErrorHandler
obj = object.__new__(ErrorHandler)
try:
    obj.error(_W())  # exception: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/ErrorHandler__fatalError__exception_as_BaseException_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_ErrorHandler__fatalError__exception_as_BaseException_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "ErrorHandler__fatalError__exception_as_BaseException_wrong"
# subject = "xml.dom.pulldom.ErrorHandler.fatalError(exception: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.ErrorHandler.fatalError(exception: BaseException); call it with the wrong type.

typeshed contract: exception is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import ErrorHandler
obj = object.__new__(ErrorHandler)
try:
    obj.fatalError(_W())  # exception: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/ErrorHandler__warning__exception_as_BaseException_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_ErrorHandler__warning__exception_as_BaseException_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "ErrorHandler__warning__exception_as_BaseException_wrong"
# subject = "xml.dom.pulldom.ErrorHandler.warning(exception: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.ErrorHandler.warning(exception: BaseException); call it with the wrong type.

typeshed contract: exception is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import ErrorHandler
obj = object.__new__(ErrorHandler)
try:
    obj.warning(_W())  # exception: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__buildDocument__uri_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__buildDocument__uri_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__buildDocument__uri_as_typed_wrong"
# subject = "xml.dom.pulldom.PullDOM.buildDocument(uri: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.buildDocument(uri: typed); call it with the wrong type.

typeshed contract: uri is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.buildDocument(_W(), None)  # uri: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__characters__chars_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__characters__chars_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__characters__chars_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.characters(chars: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.characters(chars: str); call it with the wrong type.

typeshed contract: chars is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.characters(12345)  # chars: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__comment__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__comment__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__comment__s_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.comment(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.comment(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.comment(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__endElementNS__name_as__NSName_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__endElementNS__name_as__NSName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__endElementNS__name_as__NSName_wrong"
# subject = "xml.dom.pulldom.PullDOM.endElementNS(name: _NSName)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.endElementNS(name: _NSName); call it with the wrong type.

typeshed contract: name is _NSName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.endElementNS(_W(), None)  # name: _NSName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__endElement__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__endElement__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__endElement__name_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.endElement(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.endElement(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.endElement(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__endPrefixMapping__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__endPrefixMapping__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__endPrefixMapping__prefix_as_typed_wrong"
# subject = "xml.dom.pulldom.PullDOM.endPrefixMapping(prefix: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.endPrefixMapping(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.endPrefixMapping(_W())  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__ignorableWhitespace__chars_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__ignorableWhitespace__chars_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__ignorableWhitespace__chars_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.ignorableWhitespace(chars: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.ignorableWhitespace(chars: str); call it with the wrong type.

typeshed contract: chars is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.ignorableWhitespace(12345)  # chars: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__init__documentFactory_as__DocumentFactory_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__init__documentFactory_as__DocumentFactory_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__init__documentFactory_as__DocumentFactory_wrong"
# subject = "xml.dom.pulldom.PullDOM.__init__(documentFactory: _DocumentFactory)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.__init__(documentFactory: _DocumentFactory); call it with the wrong type.

typeshed contract: documentFactory is _DocumentFactory. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
try:
    PullDOM(_W())  # documentFactory: _DocumentFactory <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__processingInstruction__target_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__processingInstruction__target_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__processingInstruction__target_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.processingInstruction(target: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.processingInstruction(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.processingInstruction(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__setDocumentLocator__locator_as_Locator_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__setDocumentLocator__locator_as_Locator_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__setDocumentLocator__locator_as_Locator_wrong"
# subject = "xml.dom.pulldom.PullDOM.setDocumentLocator(locator: Locator)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.setDocumentLocator(locator: Locator); call it with the wrong type.

typeshed contract: locator is Locator. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.setDocumentLocator(_W())  # locator: Locator <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__startElementNS__name_as__NSName_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__startElementNS__name_as__NSName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__startElementNS__name_as__NSName_wrong"
# subject = "xml.dom.pulldom.PullDOM.startElementNS(name: _NSName)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.startElementNS(name: _NSName); call it with the wrong type.

typeshed contract: name is _NSName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.startElementNS(_W(), None, None)  # name: _NSName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__startElement__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__startElement__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__startElement__name_as_str_wrong"
# subject = "xml.dom.pulldom.PullDOM.startElement(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.startElement(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.startElement(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/PullDOM__startPrefixMapping__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_PullDOM__startPrefixMapping__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "PullDOM__startPrefixMapping__prefix_as_typed_wrong"
# subject = "xml.dom.pulldom.PullDOM.startPrefixMapping(prefix: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.PullDOM.startPrefixMapping(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import PullDOM
obj = object.__new__(PullDOM)
try:
    obj.startPrefixMapping(_W(), "")  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/SAX2DOM__characters__chars_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_SAX2DOM__characters__chars_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "SAX2DOM__characters__chars_as_str_wrong"
# subject = "xml.dom.pulldom.SAX2DOM.characters(chars: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.SAX2DOM.characters(chars: str); call it with the wrong type.

typeshed contract: chars is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import SAX2DOM
obj = object.__new__(SAX2DOM)
try:
    obj.characters(12345)  # chars: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/SAX2DOM__ignorableWhitespace__chars_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_SAX2DOM__ignorableWhitespace__chars_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "SAX2DOM__ignorableWhitespace__chars_as_str_wrong"
# subject = "xml.dom.pulldom.SAX2DOM.ignorableWhitespace(chars: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.SAX2DOM.ignorableWhitespace(chars: str); call it with the wrong type.

typeshed contract: chars is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import SAX2DOM
obj = object.__new__(SAX2DOM)
try:
    obj.ignorableWhitespace(12345)  # chars: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/SAX2DOM__processingInstruction__target_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_SAX2DOM__processingInstruction__target_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "SAX2DOM__processingInstruction__target_as_str_wrong"
# subject = "xml.dom.pulldom.SAX2DOM.processingInstruction(target: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.SAX2DOM.processingInstruction(target: str); call it with the wrong type.

typeshed contract: target is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import SAX2DOM
obj = object.__new__(SAX2DOM)
try:
    obj.processingInstruction(12345, "")  # target: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/SAX2DOM__startElementNS__name_as__NSName_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_SAX2DOM__startElementNS__name_as__NSName_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "SAX2DOM__startElementNS__name_as__NSName_wrong"
# subject = "xml.dom.pulldom.SAX2DOM.startElementNS(name: _NSName)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.SAX2DOM.startElementNS(name: _NSName); call it with the wrong type.

typeshed contract: name is _NSName. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import SAX2DOM
obj = object.__new__(SAX2DOM)
try:
    obj.startElementNS(_W(), None, None)  # name: _NSName <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/SAX2DOM__startElement__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_SAX2DOM__startElement__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "SAX2DOM__startElement__name_as_str_wrong"
# subject = "xml.dom.pulldom.SAX2DOM.startElement(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.SAX2DOM.startElement(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import SAX2DOM
obj = object.__new__(SAX2DOM)
try:
    obj.startElement(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/parseString__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_parseString__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "parseString__string_as_str_wrong"
# subject = "xml.dom.pulldom.parseString(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.parseString(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xml.dom.pulldom import parseString
try:
    parseString(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xml_dom_pulldom/parse__stream_or_string_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xml_dom_pulldom_parse__stream_or_string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_pulldom"
# dimension = "type"
# case = "parse__stream_or_string_as_typed_wrong"
# subject = "xml.dom.pulldom.parse(stream_or_string: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xml/dom/pulldom.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xml.dom.pulldom.parse(stream_or_string: typed); call it with the wrong type.

typeshed contract: stream_or_string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xml.dom.pulldom import parse
try:
    parse(_W())  # stream_or_string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
