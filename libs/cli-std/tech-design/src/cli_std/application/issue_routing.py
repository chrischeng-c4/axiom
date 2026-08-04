from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from enum import Enum

from cli_std.domain.errors import MalformedRepo
from cli_std.domain.issue_body import (
    comment_payload,
    issue_payload,
    prefilled_url,
    report_labels,
)
from cli_std.domain.tool_identity import ToolInfo
from cli_std.infrastructure.courier import (
    bearer_header,
    courier_comment_url,
    courier_create_url,
    courier_search_url,
    courier_view_url,
    github_search_url,
    github_view_url,
    split_repo,
)


class Route(Enum):
    COURIER = "courier"
    DIRECT = "direct"


@dataclass(frozen=True)
class RequestPlan:
    route: Route
    method: str
    url: str
    headers: tuple[tuple[str, str], ...]
    body: Mapping[str, object] | None


@dataclass(frozen=True)
class BrowserFallback:
    url: str


GITHUB_ACCEPT: tuple[str, str] = ("Accept", "application/vnd.github+json")


def choose_route(courier_url: str | None) -> Route:
    if courier_url is not None:
        return Route.COURIER
    return Route.DIRECT


def courier_headers(
    courier_token: str | None,
) -> tuple[tuple[str, str], ...]:
    if courier_token is not None:
        return (bearer_header(courier_token),)
    return ()


def github_headers(
    github_token: str | None,
) -> tuple[tuple[str, str], ...]:
    if github_token is not None:
        return (GITHUB_ACCEPT, bearer_header(github_token))
    return (GITHUB_ACCEPT,)


def build_search_query(
    tool: ToolInfo, repo: str, route: Route, state: str, text: str | None
) -> str:
    if route == Route.COURIER:
        q = f'label:"{tool.issue_label()}"'
    else:
        q = f'repo:{repo} is:issue label:"{tool.issue_label()}"'
        if state != "all":
            q += f" state:{state}"

    if text is not None and text.strip() != "":
        q += " " + text.strip()
    return q


def plan_search(
    tool: ToolInfo,
    repo: str,
    courier_url: str | None,
    courier_token: str | None,
    github_token: str | None,
    state: str,
    text: str | None,
    limit: int,
) -> RequestPlan | MalformedRepo:
    s_repo = split_repo(repo)
    if isinstance(s_repo, MalformedRepo):
        return s_repo
    owner, name = s_repo

    route = choose_route(courier_url)
    q = build_search_query(tool, repo, route, state, text)

    if route == Route.COURIER:
        assert courier_url is not None
        url = courier_search_url(courier_url, owner, name, state, q, limit)
        return RequestPlan(
            route=Route.COURIER,
            method="GET",
            url=url,
            headers=courier_headers(courier_token),
            body=None,
        )
    else:
        url = github_search_url(q, limit)
        return RequestPlan(
            route=Route.DIRECT,
            method="GET",
            url=url,
            headers=github_headers(github_token),
            body=None,
        )


def plan_view(
    tool: ToolInfo,
    repo: str,
    courier_url: str | None,
    courier_token: str | None,
    github_token: str | None,
    number: int,
) -> RequestPlan | MalformedRepo:
    s_repo = split_repo(repo)
    if isinstance(s_repo, MalformedRepo):
        return s_repo
    owner, name = s_repo

    route = choose_route(courier_url)
    if route == Route.COURIER:
        assert courier_url is not None
        url = courier_view_url(courier_url, owner, name, number)
        return RequestPlan(
            route=Route.COURIER,
            method="GET",
            url=url,
            headers=courier_headers(courier_token),
            body=None,
        )
    else:
        url = github_view_url(repo, number)
        return RequestPlan(
            route=Route.DIRECT,
            method="GET",
            url=url,
            headers=github_headers(github_token),
            body=None,
        )


def plan_create(
    tool: ToolInfo,
    repo: str,
    courier_url: str | None,
    courier_token: str | None,
    github_token: str | None,
    title: str,
    body: str,
    labels: Sequence[str],
) -> RequestPlan | BrowserFallback | MalformedRepo:
    s_repo = split_repo(repo)
    if isinstance(s_repo, MalformedRepo):
        return s_repo
    owner, name = s_repo

    r_labels = report_labels(tool, labels)
    payload = issue_payload(title, body, r_labels)

    route = choose_route(courier_url)
    if route == Route.COURIER:
        assert courier_url is not None
        url = courier_create_url(courier_url, owner, name)
        return RequestPlan(
            route=Route.COURIER,
            method="POST",
            url=url,
            headers=courier_headers(courier_token),
            body=payload,
        )

    if github_token is None:
        return BrowserFallback(prefilled_url(repo, title, body, r_labels))

    url = f"https://api.github.com/repos/{repo}/issues"
    return RequestPlan(
        route=Route.DIRECT,
        method="POST",
        url=url,
        headers=github_headers(github_token),
        body=payload,
    )


def plan_comment(
    tool: ToolInfo,
    repo: str,
    courier_url: str | None,
    courier_token: str | None,
    github_token: str | None,
    number: int,
    body: str,
) -> tuple[RequestPlan, ...] | BrowserFallback | MalformedRepo:
    s_repo = split_repo(repo)
    if isinstance(s_repo, MalformedRepo):
        return s_repo
    owner, name = s_repo

    route = choose_route(courier_url)
    if route == Route.COURIER:
        assert courier_url is not None
        url = courier_comment_url(courier_url, owner, name, number)
        req = RequestPlan(
            route=Route.COURIER,
            method="POST",
            url=url,
            headers=courier_headers(courier_token),
            body=comment_payload(body),
        )
        return (req,)

    if github_token is None:
        issue_url = f"https://github.com/{repo}/issues/{number}"
        return BrowserFallback(issue_url)

    patch_req = RequestPlan(
        route=Route.DIRECT,
        method="PATCH",
        url=f"https://api.github.com/repos/{repo}/issues/{number}",
        headers=github_headers(github_token),
        body={"state": "open"},
    )
    post_req = RequestPlan(
        route=Route.DIRECT,
        method="POST",
        url=f"https://api.github.com/repos/{repo}/issues/{number}/comments",
        headers=github_headers(github_token),
        body=comment_payload(body),
    )
    return (patch_req, post_req)
