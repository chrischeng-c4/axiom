"""Behavior contract for third-party jinja2 package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import jinja2  # type: ignore[import]

# Rule 1: from_string renders variable substitution
_env1 = jinja2.Environment()
_tmpl1 = _env1.from_string("Hello, {{ name }}!")
_r1 = _tmpl1.render(name="Alice")
assert _r1 == "Hello, Alice!", f"render = {_r1!r}"

# Rule 2: DictLoader provides named templates
_env2 = jinja2.Environment(
    loader=jinja2.DictLoader({
        "greet.html": "Dear {{ user }},",
        "bye.html": "Goodbye {{ user }}.",
    })
)
_tmpl2a = _env2.get_template("greet.html")
assert _tmpl2a.render(user="Bob") == "Dear Bob,", \
    f"greet render = {_tmpl2a.render(user='Bob')!r}"
_tmpl2b = _env2.get_template("bye.html")
assert _tmpl2b.render(user="Bob") == "Goodbye Bob.", \
    f"bye render = {_tmpl2b.render(user='Bob')!r}"

# Rule 3: TemplateNotFound raised for missing template
_env3 = jinja2.Environment(loader=jinja2.DictLoader({}))
_raised3 = False
try:
    _env3.get_template("missing.html")
except jinja2.TemplateNotFound:
    _raised3 = True
assert _raised3, "TemplateNotFound raised"

# Rule 4: Template if/else conditional rendering
_env4 = jinja2.Environment()
_tmpl4 = _env4.from_string(
    "{% if score >= 50 %}pass{% else %}fail{% endif %}"
)
assert _tmpl4.render(score=80) == "pass", f"pass = {_tmpl4.render(score=80)!r}"
assert _tmpl4.render(score=30) == "fail", f"fail = {_tmpl4.render(score=30)!r}"

# Rule 5: Template for loop
_env5 = jinja2.Environment()
_tmpl5 = _env5.from_string("{% for x in items %}{{ x }},{% endfor %}")
_r5 = _tmpl5.render(items=[1, 2, 3])
assert _r5 == "1,2,3,", f"loop render = {_r5!r}"

# Rule 6: filter |upper applied
_env6 = jinja2.Environment()
_tmpl6 = _env6.from_string("{{ name|upper }}")
_r6 = _tmpl6.render(name="world")
assert _r6 == "WORLD", f"upper filter = {_r6!r}"

# Rule 7: Autoescape escapes HTML entities
_env7 = jinja2.Environment(autoescape=True)
_tmpl7 = _env7.from_string("{{ html }}")
_r7 = _tmpl7.render(html="<b>bold</b>")
assert "&lt;b&gt;" in _r7, f"autoescape = {_r7!r}"

# Rule 8: Module attributes are identity-stable
_env_ref = jinja2.Environment
_tmpl_ref = jinja2.Template
_fl_ref = jinja2.FileSystemLoader
_sa_ref = jinja2.select_autoescape
for _ in range(5):
    assert jinja2.Environment is _env_ref, "Environment stable"
    assert jinja2.Template is _tmpl_ref, "Template stable"
    assert jinja2.FileSystemLoader is _fl_ref, "FileSystemLoader stable"
    assert jinja2.select_autoescape is _sa_ref, "select_autoescape stable"

print("behavior OK")
