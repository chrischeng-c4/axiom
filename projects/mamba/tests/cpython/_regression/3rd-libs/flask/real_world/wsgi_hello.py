"""Construct a Flask app and exercise route dispatch via the test client.

End-user scenario: a downstream service wires a single `/hello` route
and proves the WSGI dispatch returns the expected body and status
without ever opening a socket. This is the smallest reproducible
"Flask runs unchanged" gate — anything beyond a one-route test client
walk belongs in a larger integration fixture.

DoD: this script must exit 0 under both CPython and mamba.
"""

from flask import Flask

app = Flask("real_world_wsgi_hello")


@app.route("/hello")
def hello():
    return "hello, flask", 200


# Test client = in-process WSGI call. No sockets, no network.
client = app.test_client()
response = client.get("/hello")

assert response.status_code == 200, f"unexpected status: {response.status_code}"
assert response.data == b"hello, flask", f"unexpected body: {response.data!r}"

print("ok:", response.status_code, response.data.decode("ascii"))
