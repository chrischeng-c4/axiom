"""Surface contract for third-party flask package.

# type-regime: monomorphic

Probes: flask.Flask, flask.Blueprint, flask.request, flask.__version__,
flask.jsonify, flask.url_for, flask.render_template_string.
CPython 3.12 is the oracle.
"""

import flask  # type: ignore[import]

# Core API
assert hasattr(flask, "Flask"), "Flask"
assert hasattr(flask, "Blueprint"), "Blueprint"
assert hasattr(flask, "request"), "request"
assert hasattr(flask, "__version__"), "__version__"
assert hasattr(flask, "jsonify"), "jsonify"
assert hasattr(flask, "url_for"), "url_for"
assert hasattr(flask, "render_template_string"), "render_template_string"
assert hasattr(flask, "abort"), "abort"
assert hasattr(flask, "redirect"), "redirect"
assert hasattr(flask, "g"), "g"

# Version
assert isinstance(flask.__version__, str), \
    f"version type = {type(flask.__version__)!r}"

# Flask is a class
assert callable(flask.Flask), "Flask callable"

# Flask app basic construction
_app = flask.Flask(__name__)
assert hasattr(_app, "url_map"), "app.url_map"
assert hasattr(_app, "config"), "app.config"
assert hasattr(_app, "route"), "app.route"
assert hasattr(_app, "before_request"), "app.before_request"
assert hasattr(_app, "after_request"), "app.after_request"
assert hasattr(_app, "errorhandler"), "app.errorhandler"
assert hasattr(_app, "test_client"), "app.test_client"

# Blueprint
assert callable(flask.Blueprint), "Blueprint callable"
_bp = flask.Blueprint("test_bp", __name__)
assert hasattr(_bp, "name"), "bp.name"
assert _bp.name == "test_bp", f"bp.name = {_bp.name!r}"
assert hasattr(_bp, "url_prefix"), "bp.url_prefix"
assert hasattr(_bp, "route"), "bp.route"

# jsonify is callable
assert callable(flask.jsonify), "jsonify callable"

# Module attributes stable
_f_ref = flask.Flask
assert flask.Flask is _f_ref, "Flask stable"
_b_ref = flask.Blueprint
assert flask.Blueprint is _b_ref, "Blueprint stable"
_r_ref = flask.request
assert flask.request is _r_ref, "request stable"
_v_ref = flask.__version__
assert flask.__version__ == _v_ref, "__version__ stable"

print("surface OK")
