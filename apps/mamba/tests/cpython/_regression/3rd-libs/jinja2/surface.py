"""Surface contract for third-party jinja2 package.

# type-regime: monomorphic

Probes: jinja2.Environment, jinja2.Template, jinja2.FileSystemLoader,
jinja2.select_autoescape, jinja2.TemplateNotFound, jinja2.filters.
CPython 3.12 is the oracle.
"""

import jinja2  # type: ignore[import]

# Core API
assert hasattr(jinja2, "Environment"), "Environment"
assert hasattr(jinja2, "Template"), "Template"
assert hasattr(jinja2, "FileSystemLoader"), "FileSystemLoader"
assert hasattr(jinja2, "select_autoescape"), "select_autoescape"
assert hasattr(jinja2, "TemplateNotFound"), "TemplateNotFound"
assert hasattr(jinja2, "BaseLoader"), "BaseLoader"
assert hasattr(jinja2, "DictLoader"), "DictLoader"
assert hasattr(jinja2, "TemplateSyntaxError"), "TemplateSyntaxError"
assert hasattr(jinja2, "Undefined"), "Undefined"
assert hasattr(jinja2, "ChainableUndefined"), "ChainableUndefined"

# Version
assert hasattr(jinja2, "__version__"), "__version__"
assert isinstance(jinja2.__version__, str), \
    f"version type = {type(jinja2.__version__)!r}"

# Classes are callable
assert callable(jinja2.Environment), "Environment callable"
assert callable(jinja2.Template), "Template callable"
assert callable(jinja2.FileSystemLoader), "FileSystemLoader callable"
assert callable(jinja2.DictLoader), "DictLoader callable"

# Environment construction
_env = jinja2.Environment(loader=jinja2.DictLoader({"tmpl.html": "Hello {{ name }}!"}))
assert hasattr(_env, "loader"), "env.loader"
assert hasattr(_env, "get_template"), "env.get_template"
assert hasattr(_env, "from_string"), "env.from_string"
assert hasattr(_env, "filters"), "env.filters"
assert hasattr(_env, "globals"), "env.globals"
assert hasattr(_env, "autoescape"), "env.autoescape"

# from_string produces Template
_tmpl = _env.from_string("Hello {{ name }}!")
assert hasattr(_tmpl, "render"), "tmpl.render"
assert hasattr(_tmpl, "module"), "tmpl.module"
_result = _tmpl.render(name="World")
assert isinstance(_result, str), f"render type = {type(_result)!r}"
assert _result == "Hello World!", f"render = {_result!r}"

# TemplateNotFound is an exception
assert issubclass(jinja2.TemplateNotFound, Exception), \
    "TemplateNotFound < Exception"

# select_autoescape
_ae = jinja2.select_autoescape(["html", "xml"])
assert callable(_ae), "select_autoescape returns callable"

# Module attributes stable
_env_ref = jinja2.Environment
assert jinja2.Environment is _env_ref, "Environment stable"
_tmpl_ref = jinja2.Template
assert jinja2.Template is _tmpl_ref, "Template stable"
_fl_ref = jinja2.FileSystemLoader
assert jinja2.FileSystemLoader is _fl_ref, "FileSystemLoader stable"
_sa_ref = jinja2.select_autoescape
assert jinja2.select_autoescape is _sa_ref, "select_autoescape stable"

print("surface OK")
