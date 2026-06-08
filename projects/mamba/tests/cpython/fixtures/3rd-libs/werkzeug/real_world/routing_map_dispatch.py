"""Offline Werkzeug routing dispatch (Flask's underlying URL router).

End-user scenario: Flask delegates every URL match to Werkzeug's
`Map` / `Rule` machinery. This fixture exercises the router directly
without spinning up a WSGI app: build a tiny `Map`, bind it to a
fake host, and assert that match / build / 404 / typed-converter
behavior is deterministic. No sockets, no network.

DoD: this script must exit 0 under both CPython and mamba.
"""

from werkzeug.routing import Map, Rule
from werkzeug.exceptions import NotFound

# -- 1. Build a router covering the four common Flask-style shapes ----------

url_map = Map([
    Rule("/", endpoint="index"),
    Rule("/hello", endpoint="hello"),
    Rule("/users/<int:user_id>", endpoint="user_detail"),
    Rule("/posts/<string:slug>", endpoint="post_detail"),
])

# server_name = stable string so URL building stays deterministic across runs.
adapter = url_map.bind("localhost", url_scheme="http")

# -- 2. Match the static routes ---------------------------------------------

endpoint, args = adapter.match("/")
assert endpoint == "index", f"GET / must map to 'index', got {endpoint!r}"
assert args == {}, f"GET / must yield no args, got {args!r}"

endpoint, args = adapter.match("/hello")
assert endpoint == "hello", f"GET /hello must map to 'hello', got {endpoint!r}"
assert args == {}, f"GET /hello must yield no args, got {args!r}"

# -- 3. Match the typed-converter routes ------------------------------------

endpoint, args = adapter.match("/users/42")
assert endpoint == "user_detail", f"GET /users/42 must map to 'user_detail', got {endpoint!r}"
assert args == {"user_id": 42}, f"int converter must yield int 42, got {args!r}"
assert isinstance(args["user_id"], int), (
    f"int converter must produce int, got {type(args['user_id']).__name__}"
)

endpoint, args = adapter.match("/posts/hello-world")
assert endpoint == "post_detail", (
    f"GET /posts/hello-world must map to 'post_detail', got {endpoint!r}"
)
assert args == {"slug": "hello-world"}, (
    f"string converter must yield 'hello-world', got {args!r}"
)
assert isinstance(args["slug"], str), (
    f"string converter must produce str, got {type(args['slug']).__name__}"
)

# -- 4. 404 path: unknown URL must raise NotFound ---------------------------

raised: bool = False
try:
    adapter.match("/does-not-exist")
except NotFound:
    raised = True
assert raised, "unknown path must raise werkzeug.exceptions.NotFound"

# -- 5. Type-mismatch on int converter falls through to 404 -----------------

raised = False
try:
    adapter.match("/users/not-a-number")
except NotFound:
    raised = True
assert raised, "non-int slug into <int:user_id> must raise NotFound (no fallback)"

# -- 6. URL building round-trips back to the matched path -------------------

# build() is the inverse of match(); a routing bug that breaks one half
# almost always breaks the other.
assert adapter.build("index") == "/", f"build(index) must yield '/', got {adapter.build('index')!r}"
assert adapter.build("hello") == "/hello", (
    f"build(hello) must yield '/hello', got {adapter.build('hello')!r}"
)
assert adapter.build("user_detail", {"user_id": 7}) == "/users/7", (
    f"build(user_detail) must yield '/users/7', got {adapter.build('user_detail', {'user_id': 7})!r}"
)
assert adapter.build("post_detail", {"slug": "intro"}) == "/posts/intro", (
    f"build(post_detail) must yield '/posts/intro', "
    f"got {adapter.build('post_detail', {'slug': 'intro'})!r}"
)

print("ok: routes matched and rebuilt cleanly")
