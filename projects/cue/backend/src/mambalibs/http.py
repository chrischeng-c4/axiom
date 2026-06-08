"""Httpkit-shaped preview of Cue's future Mamba HTTP surface."""

from __future__ import annotations

from http import HTTPStatus
from typing import Any, Callable


Handler = Callable[..., Any]


class App:
    """Small route registry matching the subset Cue uses today."""

    def __init__(self, **metadata: Any) -> None:
        self.metadata = metadata
        self._handlers: dict[tuple[str, str], Handler] = {}
        self._mounts: list[tuple[str, Any, str | None]] = []
        self._middleware: list[tuple[type[Any], dict[str, Any]]] = []
        self._events: dict[str, list[Handler]] = {}

    def add_middleware(self, middleware: type[Any], **kwargs: Any) -> None:
        self._middleware.append((middleware, kwargs))

    def mount(self, path: str, app: Any, name: str | None = None) -> None:
        self._mounts.append((path, app, name))

    def include_router(self, router: "Router", prefix: str = "") -> None:
        for (method, path), handler in router._handlers.items():
            self._handlers[(method, f"{prefix}{path}")] = handler

    def on_event(self, event: str) -> Callable[[Handler], Handler]:
        def wrapper(fn: Handler) -> Handler:
            self._events.setdefault(event, []).append(fn)
            return fn

        return wrapper

    def get(self, path: str) -> Callable[[Handler], Handler]:
        return self._route("GET", path)

    def post(self, path: str) -> Callable[[Handler], Handler]:
        return self._route("POST", path)

    def put(self, path: str) -> Callable[[Handler], Handler]:
        return self._route("PUT", path)

    def patch(self, path: str) -> Callable[[Handler], Handler]:
        return self._route("PATCH", path)

    def delete(self, path: str) -> Callable[[Handler], Handler]:
        return self._route("DELETE", path)

    def _route(self, method: str, path: str) -> Callable[[Handler], Handler]:
        def wrapper(fn: Handler) -> Handler:
            self._handlers[(method, path)] = fn
            return fn

        return wrapper


class Router(App):
    """Router uses the same route registry shape as App for now."""


class CORSMiddleware:
    """Configuration marker consumed by the future httpkit runtime."""


class StaticFiles:
    """Static-file mount marker consumed by the future httpkit runtime."""

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        self.args = args
        self.kwargs = kwargs


class HTTPException(Exception):
    def __init__(self, status_code: int | HTTPStatus, detail: Any = None) -> None:
        self.status_code = int(status_code)
        self.detail = detail
        super().__init__(detail)


class Depends:
    def __init__(self, dependency: Callable[..., Any] | None = None, **metadata: Any) -> None:
        self.dependency = dependency
        self.metadata = metadata


class Query:
    def __init__(self, default: Any = None, **metadata: Any) -> None:
        self.default = default
        self.metadata = metadata


class Body(Query):
    pass


class Header(Query):
    pass


class BackgroundTasks:
    def __init__(self) -> None:
        self.tasks: list[tuple[Callable[..., Any], tuple[Any, ...], dict[str, Any]]] = []

    def add_task(self, fn: Callable[..., Any], *args: Any, **kwargs: Any) -> None:
        self.tasks.append((fn, args, kwargs))


class Request:
    async def json(self) -> Any:
        return {}


RequestContext = Request
FastAPI = App
