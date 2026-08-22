use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_adjacent_no_space.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_adjacent_no_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_adjacent_no_space"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: adjacent attributes with no separating space still split (<a width="100%"cellspacing=0>)"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of('<a width="100%"cellspacing=0>') == [
    ("width", "100%"), ("cellspacing", "0")
], attrs_of('<a width="100%"cellspacing=0>')

print("attr_adjacent_no_space OK")
"###);
    assert_output(&out, r###"attr_adjacent_no_space OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_bare_equals_yields_empty.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_bare_equals_yields_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_bare_equals_yields_empty"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a bare '=' with no following value yields an empty-string attribute value"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<a v=>") == [("v", "")], attrs_of("<a v=>")

print("attr_bare_equals_yields_empty OK")
"###);
    assert_output(&out, r###"attr_bare_equals_yields_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_empty_string_value_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_empty_string_value_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_empty_string_value_preserved"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: empty quoted attribute values are preserved as '' and not collapsed to None"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<a b='' c=\"\">") == [("b", ""), ("c", "")], attrs_of("<a b='' c=\"\">")

print("attr_empty_string_value_preserved OK")
"###);
    assert_output(&out, r###"attr_empty_string_value_preserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_entity_expanded_in_value.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_entity_expanded_in_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_entity_expanded_in_value"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: entity references inside an attribute value are expanded (&amp;&gt;&lt;&quot;&apos; -> &><"')"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<a b='&amp;&gt;&lt;&quot;&apos;'>") == [
    ("b", "&><\"'")
], attrs_of("<a b='&amp;&gt;&lt;&quot;&apos;'>")

print("attr_entity_expanded_in_value OK")
"###);
    assert_output(&out, r###"attr_entity_expanded_in_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_funky_name_characters.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_funky_name_characters() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_funky_name_characters"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: funky-but-legal attribute name characters (a.b c:d e-f) survive intact"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<a a.b='v' c:d=v e-f=v>") == [
    ("a.b", "v"), ("c:d", "v"), ("e-f", "v")
], attrs_of("<a a.b='v' c:d=v e-f=v>")

print("attr_funky_name_characters OK")
"###);
    assert_output(&out, r###"attr_funky_name_characters OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_mixed_quoting_and_valueless.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_mixed_quoting_and_valueless() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_mixed_quoting_and_valueless"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: single, double, and unquoted values parse equally and a valueless flag attribute gets value None"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<a b='v' c=\"v\" d=v e>") == [
    ("b", "v"), ("c", "v"), ("d", "v"), ("e", None)
], attrs_of("<a b='v' c=\"v\" d=v e>")

print("attr_mixed_quoting_and_valueless OK")
"###);
    assert_output(&out, r###"attr_mixed_quoting_and_valueless OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/attr_non_ascii_unquoted_value.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_attr_non_ascii_unquoted_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "attr_non_ascii_unquoted_value"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: non-ASCII unquoted attribute values are kept verbatim (<img src=/foo/bar.png alt=中文>)"""
from html.parser import HTMLParser



class AttrParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.attrs = None

    def handle_starttag(self, tag, attrs):
        self.attrs = attrs


def attrs_of(source):
    p = AttrParser()
    p.feed(source)
    return p.attrs

assert attrs_of("<img src=/foo/bar.png alt=\u4e2d\u6587>") == [
    ("src", "/foo/bar.png"), ("alt", "\u4e2d\u6587")
], attrs_of("<img src=/foo/bar.png alt=\u4e2d\u6587>")

print("attr_non_ascii_unquoted_value OK")
"###);
    assert_output(&out, r###"attr_non_ascii_unquoted_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/bogus_decl_becomes_comment.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_bogus_decl_becomes_comment() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "bogus_decl_becomes_comment"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a bogus markup declaration <!spacer type="block"> is reported as a comment, not a decl"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of('<!spacer type="block">') == [
    ("comment", 'spacer type="block"')
], events_of('<!spacer type="block">')

print("bogus_decl_becomes_comment OK")
"###);
    assert_output(&out, r###"bogus_decl_becomes_comment OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/cdata_marked_section_unknown_decl.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_cdata_marked_section_unknown_decl() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "cdata_marked_section_unknown_decl"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a CDATA-looking marked section inside foreign content fires unknown_decl with 'CDATA[raw'"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of('<svg><text y="100"><![CDATA[raw]]></text></svg>') == [
    ("starttag", "svg", []),
    ("starttag", "text", [("y", "100")]),
    ("unknown decl", "CDATA[raw"),
    ("endtag", "text"),
    ("endtag", "svg"),
], events_of('<svg><text y="100"><![CDATA[raw]]></text></svg>')

print("cdata_marked_section_unknown_decl OK")
"###);
    assert_output(&out, r###"cdata_marked_section_unknown_decl OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/comment_content_captured.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_comment_content_captured() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "comment_content_captured"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: handle_comment captures comment text verbatim; <!-- comment text --> yields the single comment ' comment text '"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<!-- comment text -->")
comments = [e[1] for e in p.events if e[0] == "comment"]
assert len(comments) == 1, comments
assert comments[0] == " comment text ", repr(comments[0])

print("comment_content_captured OK")
"###);
    assert_output(&out, r###"comment_content_captured OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/comment_interior_markup_not_parsed.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_comment_interior_markup_not_parsed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "comment_interior_markup_not_parsed"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: comment content is delivered verbatim and interior markup is not parsed; <!-- <b>not a tag</b> --> is one comment"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("<!-- <b>not a tag</b> -->") == [
    ("comment", " <b>not a tag</b> ")
], events_of("<!-- <b>not a tag</b> -->")

print("comment_interior_markup_not_parsed OK")
"###);
    assert_output(&out, r###"comment_interior_markup_not_parsed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/convert_charrefs_named_entities.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_convert_charrefs_named_entities() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "convert_charrefs_named_entities"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: convert_charrefs=True converts named entities &amp; &lt; &gt; to & < > in the delivered data"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec(convert_charrefs=True)
p.feed("<p>&amp; &lt; &gt; &quot;</p>")
data = "".join(e[1] for e in p.events if e[0] == "data")
assert "&" in data, data
assert "<" in data, data
assert ">" in data, data

print("convert_charrefs_named_entities OK")
"###);
    assert_output(&out, r###"convert_charrefs_named_entities OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/convert_charrefs_numeric_entities.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_convert_charrefs_numeric_entities() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "convert_charrefs_numeric_entities"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: convert_charrefs=True converts decimal &#65;&#66; and hex &#x43; numeric character references to A, B, C"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec(convert_charrefs=True)
p.feed("<p>&#65;&#66;&#x43;</p>")  # A, B, C
data = "".join(e[1] for e in p.events if e[0] == "data")
assert "A" in data, data
assert "B" in data, data
assert "C" in data, data

print("convert_charrefs_numeric_entities OK")
"###);
    assert_output(&out, r###"convert_charrefs_numeric_entities OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/data_captures_text_content.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_data_captures_text_content() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "data_captures_text_content"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: handle_data captures the text content between tags; <p>Hello World</p> yields data 'Hello World'"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<p>Hello World</p>")
data = [e[1] for e in p.events if e[0] == "data"]
assert "Hello World" in data, data

print("data_captures_text_content OK")
"###);
    assert_output(&out, r###"data_captures_text_content OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/doctype_decl_event.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_doctype_decl_event() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "doctype_decl_event"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a DOCTYPE declaration surfaces through handle_decl as 'DOCTYPE html'"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("<!DOCTYPE html>") == [("decl", "DOCTYPE html")], events_of("<!DOCTYPE html>")

print("doctype_decl_event OK")
"###);
    assert_output(&out, r###"doctype_decl_event OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/getpos_returns_line_col_tuple.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_getpos_returns_line_col_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "getpos_returns_line_col_tuple"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: getpos() returns a (line, col) tuple of two ints after feeding markup"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<div>")
pos = p.getpos()
assert isinstance(pos, tuple), type(pos)
assert len(pos) == 2, pos
assert isinstance(pos[0], int) and isinstance(pos[1], int), pos

print("getpos_returns_line_col_tuple OK")
"###);
    assert_output(&out, r###"getpos_returns_line_col_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/html_escape_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_html_escape_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "html_escape_roundtrip"
# subject = "html.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.escape: html.escape('<b>x</b>') == '&lt;b&gt;x&lt;/b&gt;' and html.unescape round-trips it back"""
import html


escaped = html.escape("<b>x</b>")
assert escaped == "&lt;b&gt;x&lt;/b&gt;", escaped
assert html.unescape(escaped) == "<b>x</b>", html.unescape(escaped)

print("html_escape_roundtrip OK")
"###);
    assert_output(&out, r###"html_escape_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/incremental_chunked_feed.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_incremental_chunked_feed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "incremental_chunked_feed"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: feeding <div><span>hello</span></div> in 3-char chunks then close() yields the same div/span tags and 'hello' data"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
html = "<div><span>hello</span></div>"
for i in range(0, len(html), 3):
    p.feed(html[i:i + 3])
p.close()
starts = [e[1] for e in p.events if e[0] == "start"]
assert "div" in starts and "span" in starts, starts
text = "".join(e[1] for e in p.events if e[0] == "data")
assert "hello" in text, text

print("incremental_chunked_feed OK")
"###);
    assert_output(&out, r###"incremental_chunked_feed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/processing_instruction_event.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_processing_instruction_event() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "processing_instruction_event"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a processing instruction <?processing instruction> surfaces through handle_pi"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("<?processing instruction>") == [
    ("pi", "processing instruction")
], events_of("<?processing instruction>")

print("processing_instruction_event OK")
"###);
    assert_output(&out, r###"processing_instruction_event OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/reset_then_refeed.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_reset_then_refeed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "reset_then_refeed"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: reset() returns the parser to a clean state so a subsequent feed of <div>new</div> is parsed correctly"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<span>text</span>")
p.reset()
p.events.clear()
p.feed("<div>new</div>")
starts = [e[1] for e in p.events if e[0] == "start"]
assert "div" in starts, p.events
data = [e[1] for e in p.events if e[0] == "data"]
assert "new" in data, p.events

print("reset_then_refeed OK")
"###);
    assert_output(&out, r###"reset_then_refeed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/self_closing_tag_recognized.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_self_closing_tag_recognized() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "self_closing_tag_recognized"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: self-closing void tags <br/><img .../><input .../> are reported as start tags br/img/input"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<br/><img src='x.png'/><input type='text'/>")
starts = [e[1] for e in p.events if e[0] in ("start", "startendtag")]
assert "br" in starts, starts
assert "img" in starts, starts
assert "input" in starts, starts

print("self_closing_tag_recognized OK")
"###);
    assert_output(&out, r###"self_closing_tag_recognized OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/startendtag_for_self_closing.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_startendtag_for_self_closing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "startendtag_for_self_closing"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: the self-closing form <br/> fires handle_startendtag (not separate start+end events) by default"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("<br/>") == [("startendtag", "br", [])], events_of("<br/>")

print("startendtag_for_self_closing OK")
"###);
    assert_output(&out, r###"startendtag_for_self_closing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/starttag_reports_tag_and_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_starttag_reports_tag_and_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "starttag_reports_tag_and_attrs"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: handle_starttag reports the tag name and an attr list; <a href=... id=...> yields tag 'a' with both attributes"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed('<a href="http://example.com" id="link">text</a>')
starts = [e for e in p.events if e[0] == "start"]
assert len(starts) == 1, starts
assert starts[0][1] == "a", starts[0][1]
attrs = dict(starts[0][2])
assert attrs["href"] == "http://example.com", attrs["href"]
assert attrs["id"] == "link", attrs["id"]

print("starttag_reports_tag_and_attrs OK")
"###);
    assert_output(&out, r###"starttag_reports_tag_and_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tag_names_lowercased.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tag_names_lowercased() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tag_names_lowercased"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: tag names are lowercased (HTML is case-insensitive); <DIV><P>...</P></DIV> reports div/p in all start and end events"""
from html.parser import HTMLParser



class Rec(HTMLParser):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("start", tag, list(attrs)))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, list(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("end", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))

    def handle_entityref(self, name):
        self.events.append(("entityref", name))

    def handle_charref(self, name):
        self.events.append(("charref", name))

p = Rec()
p.feed("<DIV><P>text</P></DIV>")
names = [e[1] for e in p.events if e[0] in ("start", "end")]
assert all(t == t.lower() for t in names), names
assert "div" in names and "p" in names, names

print("tag_names_lowercased OK")
"###);
    assert_output(&out, r###"tag_names_lowercased OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tolerant_bad_nesting_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tolerant_bad_nesting_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tolerant_bad_nesting_preserved"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: overlapping/mismatched tags <a><b></a></b> are emitted as-is with no tree repair"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("<a><b></a></b>") == [
    ("starttag", "a", []),
    ("starttag", "b", []),
    ("endtag", "a"),
    ("endtag", "b"),
], events_of("<a><b></a></b>")

print("tolerant_bad_nesting_preserved OK")
"###);
    assert_output(&out, r###"tolerant_bad_nesting_preserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tolerant_bare_ampersands_as_data.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tolerant_bare_ampersands_as_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tolerant_bare_ampersands_as_data"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: bare ampersands in text ('this & that &') pass through as plain data"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert text_of("this & that &") == "this & that &", text_of("this & that &")

print("tolerant_bare_ampersands_as_data OK")
"###);
    assert_output(&out, r###"tolerant_bare_ampersands_as_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tolerant_bare_pointy_brackets_as_data.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tolerant_bare_pointy_brackets_as_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tolerant_bare_pointy_brackets_as_data"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a lone '<' that is not a real tag ('a < b > c') stays as data with no events lost"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert text_of("a < b > c") == "a < b > c", text_of("a < b > c")

print("tolerant_bare_pointy_brackets_as_data OK")
"###);
    assert_output(&out, r###"tolerant_bare_pointy_brackets_as_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tolerant_empty_end_tag_dropped.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tolerant_empty_end_tag_dropped() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tolerant_empty_end_tag_dropped"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: an empty end tag '</>' is silently discarded (no events)"""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("</>") == [], events_of("</>")

print("tolerant_empty_end_tag_dropped OK")
"###);
    assert_output(&out, r###"tolerant_empty_end_tag_dropped OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tolerant_end_tag_open_becomes_comment.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tolerant_end_tag_open_becomes_comment() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tolerant_end_tag_open_becomes_comment"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: '</ a>' normalizes whitespace and emits endtag 'a'."""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("</ a>") == [("endtag", "a")], events_of("</ a>")

print("tolerant_end_tag_open_becomes_comment OK")
"###);
    assert_output(&out, r###"tolerant_end_tag_open_becomes_comment OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/html_parser/tolerant_truncated_tag_at_eof_dropped.py`.
#[test]
fn test_gen_behavior_std_libs_html_parser_tolerant_truncated_tag_at_eof_dropped() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "behavior"
# case = "tolerant_truncated_tag_at_eof_dropped"
# subject = "html.parser.HTMLParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: a truncated start tag at EOF is emitted as data chunks."""
from html.parser import HTMLParser



class EventParser(HTMLParser):
    def __init__(self, **kw):
        super().__init__(convert_charrefs=False, **kw)
        self.events = []

    def handle_starttag(self, tag, attrs):
        self.events.append(("starttag", tag, attrs))

    def handle_startendtag(self, tag, attrs):
        self.events.append(("startendtag", tag, attrs))

    def handle_endtag(self, tag):
        self.events.append(("endtag", tag))

    def handle_data(self, data):
        self.events.append(("data", data))

    def handle_comment(self, data):
        self.events.append(("comment", data))

    def handle_decl(self, decl):
        self.events.append(("decl", decl))

    def handle_pi(self, data):
        self.events.append(("pi", data))

    def unknown_decl(self, data):
        self.events.append(("unknown decl", data))


def events_of(source):
    p = EventParser()
    p.feed(source)
    p.close()
    return p.events


def text_of(source):
    return "".join(e[1] for e in events_of(source) if e[0] == "data")

assert events_of("<a foo='bar'") == [("data", "<"), ("data", "a foo='bar'")], events_of("<a foo='bar'")

print("tolerant_truncated_tag_at_eof_dropped OK")
"###);
    assert_output(&out, r###"tolerant_truncated_tag_at_eof_dropped OK
"###);
}
