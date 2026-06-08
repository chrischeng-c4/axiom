---
change: cclab-api-asgi-dispatch
group: asgi-fix
date: 2026-03-26
---

# Requirements

Fix cclab.api.App ASGI dispatch so FastAPI Router endpoints work. Currently: app.get/post decorators work (root / returns 200), but include_router routes return 404 because App.__call__ uses its own routing table that doesn't match FastAPI's URL patterns with path parameters. Solution: build a FastAPI app internally and delegate ASGI __call__ to it.
