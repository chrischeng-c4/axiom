#!/usr/bin/env python3
"""Polyglot conformance worker (#442): a tiny Python worker that round-trips
through the loom schema layer over a **bidi h2c stream** — proving the no-SDK
polyglot contract.

It reads self-describing Task envelopes pushed down the stream, fetches input
from the keep URL the envelope gives (or inline), runs a handler keyed by
task_name (logic loom never sees), and sends Done back up the *same* stream. It
owns no relay/keep key schema.

Transport note (measured): httpx's `http2=True` only negotiates h2 over TLS-ALPN
— on a cleartext (h2c) endpoint it falls back to HTTP/1.1, where the bidi stream
can't multiplex. So this uses the low-level `h2` protocol lib (sans-io) over a
raw asyncio socket to speak **cleartext h2c prior-knowledge**. Any language with
an h2 library can do this; high-level HTTP clients generally cannot over h2c.

Deps: `pip install h2 httpx`.  Usage: `bidi_worker.py <schema-layer-port>`.
"""
import asyncio
import json
import struct
import sys

import h2.config
import h2.connection
import h2.events
import httpx  # keep I/O is plain h1; only the bidi consume needs h2c

HOST, PORT = "127.0.0.1", int(sys.argv[1])


def frame(obj):
    body = json.dumps(obj).encode()
    return struct.pack(">I", len(body)) + body


# Handler logic the schema layer / loom never sees — the polyglot point.
HANDLERS = {
    "echo": lambda b: b,
    "upper": lambda b: b.upper(),
    "exclaim": lambda b: b + b"!",
}


async def main():
    reader, writer = await asyncio.open_connection(HOST, PORT)
    conn = h2.connection.H2Connection(config=h2.config.H2Configuration(client_side=True))
    conn.initiate_connection()  # h2c prior-knowledge preface
    writer.write(conn.data_to_send())
    await writer.drain()

    sid = conn.get_next_available_stream_id()
    conn.send_headers(
        sid,
        [
            (":method", "POST"),
            (":authority", f"{HOST}:{PORT}"),
            (":scheme", "http"),
            (":path", "/v1/work/stream"),
            ("content-type", "application/octet-stream"),
        ],
        end_stream=False,
    )
    conn.send_data(sid, frame({"type": "subscribe", "group": "resident", "prefetch": 4}))
    writer.write(conn.data_to_send())
    await writer.drain()
    print("connected (Python h2c bidi)", flush=True)

    keep = httpx.AsyncClient()
    buf = b""
    while True:
        data = await reader.read(65536)
        if not data:
            break
        for ev in conn.receive_data(data):
            if isinstance(ev, h2.events.DataReceived):
                conn.acknowledge_received_data(ev.flow_controlled_length, ev.stream_id)
                buf += ev.data
                while len(buf) >= 4:
                    n = struct.unpack(">I", buf[:4])[0]
                    if len(buf) < 4 + n:
                        break
                    env = json.loads(buf[4 : 4 + n])
                    buf = buf[4 + n :]
                    inp = env["input"]
                    # scoped keep token (#444), only when present (empty Bearer is illegal)
                    auth = {"authorization": f"Bearer {env['token']}"} if env.get("token") else {}
                    if inp["kind"] == "inline":
                        d = bytes(inp["bytes"])
                    elif inp["kind"] == "keep_url":
                        d = (await keep.get(inp["url"], headers=auth)).content
                    else:
                        d = b""
                    out = HANDLERS.get(env["task_name"], lambda b: b)(d)
                    if len(out) <= 4096:  # small → inline; large → keep, then Done
                        conn.send_data(sid, frame({"type": "done", "id": env["id"], "result_inline": list(out)}))
                    else:
                        await keep.put(env["result_put_url"], content=out, headers=auth)
                        conn.send_data(sid, frame({"type": "done", "id": env["id"]}))
                    print(f"ran {env['task_name']} {env['id']}", flush=True)
            elif isinstance(ev, h2.events.StreamEnded):
                return
        pending = conn.data_to_send()
        if pending:
            writer.write(pending)
            await writer.drain()


if __name__ == "__main__":
    asyncio.run(main())
