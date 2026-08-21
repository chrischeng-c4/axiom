"""Behavior contract for third-party flask package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import flask  # type: ignore[import]
import json

# Rule 1: Flask app construction stores name as import_name
_app1 = flask.Flask("my_app")
assert _app1.name == "my_app", f"app.name = {_app1.name!r}"
assert _app1.config is not None, "app.config exists"
assert isinstance(_app1.config, dict), f"config type = {type(_app1.config)!r}"

# Rule 2: Route registration via @app.route adds to url_map
_app2 = flask.Flask("route_app")

@_app2.route("/hello")
def _hello2():
    return "Hello"

@_app2.route("/world")
def _world2():
    return "World"

_rules2 = [str(r) for r in _app2.url_map.iter_rules()]
assert "/hello" in _rules2, f"/hello in rules: {_rules2!r}"
assert "/world" in _rules2, f"/world in rules: {_rules2!r}"

# Rule 3: Blueprint name and url_prefix
_bp3 = flask.Blueprint("api", __name__, url_prefix="/api")
assert _bp3.name == "api", f"bp.name = {_bp3.name!r}"
assert _bp3.url_prefix == "/api", f"bp.url_prefix = {_bp3.url_prefix!r}"

# Rule 4: Flask test_client sends requests
_app4 = flask.Flask("test_app")

@_app4.route("/ping")
def _ping4():
    return "pong", 200

with _app4.test_client() as _c4:
    _resp4 = _c4.get("/ping")
    assert _resp4.status_code == 200, f"ping status = {_resp4.status_code!r}"
    _data4 = _resp4.get_data(as_text=True)
    assert "pong" in _data4, f"ping body = {_data4!r}"

# Rule 5: jsonify returns JSON response
_app5 = flask.Flask("json_app")

@_app5.route("/data")
def _data5():
    return flask.jsonify({"name": "Alice", "score": 42})

with _app5.test_client() as _c5:
    _resp5 = _c5.get("/data")
    assert _resp5.status_code == 200, f"json status = {_resp5.status_code!r}"
    _json5 = json.loads(_resp5.get_data(as_text=True))
    assert _json5["name"] == "Alice", f"json name = {_json5['name']!r}"
    assert _json5["score"] == 42, f"json score = {_json5['score']!r}"

# Rule 6: Flask config is a dict-like object
_app6 = flask.Flask("config_app")
_app6.config["MY_KEY"] = "my_value"
assert _app6.config["MY_KEY"] == "my_value", "config set/get"
assert "DEBUG" in _app6.config, "DEBUG in default config"

# Rule 7: Module attributes are identity-stable
_f_ref = flask.Flask
_b_ref = flask.Blueprint
_r_ref = flask.request
_v_ref = flask.__version__
for _ in range(5):
    assert flask.Flask is _f_ref, "Flask stable"
    assert flask.Blueprint is _b_ref, "Blueprint stable"
    assert flask.request is _r_ref, "request stable"
    assert flask.__version__ == _v_ref, "__version__ stable"

print("behavior OK")
