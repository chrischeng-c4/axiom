# gevent/greenlet to asyncio

Mamba does not ship a `gevent` or `greenlet` compatibility shim.

`import gevent` and `import greenlet` intentionally fail at import time with a
message pointing here. The supported direction is to migrate to `asyncio`,
native async libraries, and ASGI-style serving instead of carrying forward a
greenlet-based stack.

## What to remove

Delete greenlet/gevent bootstrapping such as:

```python
from gevent import monkey
monkey.patch_all()
```

Mamba does not support monkey-patching the stdlib eventing/socket stack. Use
libraries that already expose `async def` APIs and run them under `asyncio`.

## Common API swaps

`gevent.spawn(...)` becomes `asyncio.create_task(...)` inside an async context.

`gevent.joinall(tasks)` becomes `await asyncio.gather(*tasks)`.

`gevent.sleep(seconds)` becomes `await asyncio.sleep(seconds)`.

A direct shape conversion looks like this:

```python
# before
import gevent

jobs = [gevent.spawn(fetch_one, item) for item in items]
gevent.joinall(jobs)
results = [job.value for job in jobs]
```

```python
# after
import asyncio

async def main():
    jobs = [asyncio.create_task(fetch_one(item)) for item in items]
    return await asyncio.gather(*jobs)

results = asyncio.run(main())
```

## Serving stack migration

If you currently rely on Gunicorn's gevent worker class, move to an ASGI-native
stack instead:

- WSGI + gevent worker -> ASGI app + `uvicorn`
- gevent-patched socket/http clients -> async-native clients
- implicit cooperative concurrency -> explicit `await` points

For mamba-targeted code, prefer libraries that already publish asyncio/ASGI
surfaces instead of patching synchronous frameworks into cooperative mode.

## Local state and context

`greenlet.local()` usage should migrate to `contextvars.ContextVar`.

```python
# before
from greenlet import local

request_state = local()
request_state.user_id = user_id
```

```python
# after
from contextvars import ContextVar

request_user_id = ContextVar("request_user_id")
request_user_id.set(user_id)
```

`ContextVar` keeps request/task-scoped state aligned with `asyncio` task
switching instead of depending on greenlet-local storage.

## Migration checklist

1. Remove `gevent.monkey.patch_all()` and any import-time monkey patch setup.
2. Replace `gevent.spawn`/`joinall`/`sleep` patterns with `asyncio`
   task/gather/sleep equivalents.
3. Replace `greenlet.local()` with `contextvars.ContextVar`.
4. Move gevent worker deployments to ASGI/`uvicorn` or another asyncio-native
   server/process model.
5. Prefer async-native client libraries instead of gevent-patched sync clients.
