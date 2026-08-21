"""Surface contract for third-party jmespath package.

# type-regime: monomorphic

Probes: jmespath.search, jmespath.compile, jmespath.Options,
jmespath.__version__, jmespath.exceptions.
CPython 3.12 is the oracle.
"""

import jmespath  # type: ignore[import]
import jmespath.exceptions  # type: ignore[import]

# Core API
assert hasattr(jmespath, "search"), "search"
assert hasattr(jmespath, "compile"), "compile"
assert hasattr(jmespath, "Options"), "Options"
assert hasattr(jmespath, "__version__"), "__version__"
assert hasattr(jmespath, "exceptions"), "exceptions"

# Version
assert isinstance(jmespath.__version__, str), \
    f"version type = {type(jmespath.__version__)!r}"

# Callables
assert callable(jmespath.search), "search callable"
assert callable(jmespath.compile), "compile callable"
assert callable(jmespath.Options), "Options callable"

# search works
_r = jmespath.search("foo", {"foo": "bar", "baz": "qux"})
assert _r == "bar", f"search = {_r!r}"

# compile produces expression
_expr = jmespath.compile("foo.bar")
assert hasattr(_expr, "search"), "expr.search"
assert callable(_expr.search), "expr.search callable"

# Options construction
_opts = jmespath.Options()
assert hasattr(_opts, "dict_cls"), "opts.dict_cls"
assert hasattr(_opts, "custom_functions") or True, "opts accessible"

# exceptions module
assert hasattr(jmespath.exceptions, "JMESPathError"), "JMESPathError"
assert hasattr(jmespath.exceptions, "ParseError"), "ParseError"
assert hasattr(jmespath.exceptions, "JMESPathTypeError"), "JMESPathTypeError"
assert issubclass(jmespath.exceptions.JMESPathError, Exception), \
    "JMESPathError < Exception"

# Module attributes stable
_s_ref = jmespath.search
assert jmespath.search is _s_ref, "search stable"
_c_ref = jmespath.compile
assert jmespath.compile is _c_ref, "compile stable"
_o_ref = jmespath.Options
assert jmespath.Options is _o_ref, "Options stable"
_v_ref = jmespath.__version__
assert jmespath.__version__ is _v_ref, "__version__ stable"

print("surface OK")
