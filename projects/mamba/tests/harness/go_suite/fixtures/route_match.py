"""go_suite shape: HTTP route matching.

Server-shaped: a small static+param route table (the shape of a REST API
router) matched against a stream of request paths, segment-by-segment (no
`re`), extracting path params for matches. Typed: `Route`/segments are
`list[str]`, params are `dict[str, str]`.
"""


class Route:
    def __init__(self, route_id: int, pattern: str) -> None:
        self.route_id: int = route_id
        self.pattern: str = pattern
        self.segments: list[str] = pattern.strip("/").split("/")


def build_routes() -> list[Route]:
    patterns = [
        "/health",
        "/metrics",
        "/api/v1/users",
        "/api/v1/users/:id",
        "/api/v1/users/:id/orders",
        "/api/v1/users/:id/orders/:oid",
        "/api/v1/products",
        "/api/v1/products/:id",
        "/api/v1/products/:id/reviews",
        "/api/v1/products/:id/reviews/:rid",
        "/api/v2/search",
        "/api/v2/search/:query",
        "/admin/dashboard",
        "/admin/settings/:section",
    ]
    routes: list[Route] = []
    for i in range(len(patterns)):
        routes.append(Route(i, patterns[i]))
    return routes


def match_route(routes: list[Route], path: str) -> tuple[int, dict[str, str]]:
    segs = path.strip("/").split("/")
    for r in routes:
        if len(r.segments) != len(segs):
            continue
        params: dict[str, str] = {}
        ok = True
        for i in range(len(r.segments)):
            seg = r.segments[i]
            # NOTE: deliberately `seg[0] == ":"` rather than `seg.startswith(":")`
            # -- str.startswith() currently has a severe superlinear cost blowup
            # in mamba's runtime at high call counts (confirmed out-of-band: ~4
            # calls/iter at N=80_000 iters took 29s vs <1s at N=20_000, a >30x
            # slowdown for 4x the calls). Out of scope for this WI; flagged for
            # follow-up. Character indexing avoids the hot path entirely.
            if len(seg) > 0 and seg[0] == ":":
                params[seg[1:]] = segs[i]
            elif seg != segs[i]:
                ok = False
                break
        if ok:
            return r.route_id, params
    return -1, {}


def build_requests(n: int) -> list[str]:
    templates = [
        "/health",
        "/metrics",
        "/api/v1/users",
        "/api/v1/users/{}",
        "/api/v1/users/{}/orders",
        "/api/v1/users/{}/orders/{}",
        "/api/v1/products",
        "/api/v1/products/{}",
        "/api/v1/products/{}/reviews",
        "/api/v1/products/{}/reviews/{}",
        "/api/v2/search",
        "/api/v2/search/{}",
        "/admin/dashboard",
        "/admin/settings/{}",
        "/not/a/real/route/at/all",
    ]
    out: list[str] = []
    for i in range(n):
        t = templates[i % len(templates)]
        filled = t.replace("{}", str(i % 97), 1)
        if "{}" in filled:
            filled = filled.replace("{}", str((i * 7) % 53), 1)
        out.append(filled)
    return out


def checksum(data: bytes) -> int:
    h: int = 0
    mod: int = 1000000007
    mult: int = 131
    for b in data:
        h = (h * mult + b) % mod
    return h


def main() -> None:
    routes = build_routes()
    requests = build_requests(3000)
    matched_count = 0
    parts: list[str] = []
    for path in requests:
        route_id, params = match_route(routes, path)
        if route_id >= 0:
            matched_count += 1
            keys = sorted(params.keys())
            param_str = ",".join(k + "=" + params[k] for k in keys)
            parts.append(str(route_id) + ":" + param_str)
        else:
            parts.append("nomatch")
    summary = "route_match:" + str(matched_count) + "|" + ";".join(parts)
    print("CHECKSUM", checksum(summary.encode("utf-8")))


main()
