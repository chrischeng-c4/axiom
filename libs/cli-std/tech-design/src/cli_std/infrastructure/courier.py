from __future__ import annotations

from cli_std.domain.errors import MalformedRepo
from cli_std.domain.issue_body import percent_encode_query


def split_repo(repo: str) -> tuple[str, str] | MalformedRepo:
    if "/" not in repo:
        return MalformedRepo(repo)
    owner, name = repo.split("/", 1)
    if not owner or not name:
        return MalformedRepo(repo)
    return (owner, name)


def courier_search_url(
    courier_url: str, owner: str, name: str, state: str, q: str, limit: int
) -> str:
    base = courier_url.rstrip("/")
    return f"{base}/v1/issues/{owner}/{name}?state={percent_encode_query(state)}&q={percent_encode_query(q)}&limit={limit}"


def courier_view_url(
    courier_url: str, owner: str, name: str, number: int
) -> str:
    base = courier_url.rstrip("/")
    return f"{base}/v1/issues/{owner}/{name}/{number}"


def courier_create_url(courier_url: str, owner: str, name: str) -> str:
    base = courier_url.rstrip("/")
    return f"{base}/v1/issues/{owner}/{name}"


def courier_comment_url(
    courier_url: str, owner: str, name: str, number: int
) -> str:
    base = courier_url.rstrip("/")
    return f"{base}/v1/issues/{owner}/{name}/{number}/comments"


def github_search_url(q: str, limit: int) -> str:
    return f"https://api.github.com/search/issues?q={percent_encode_query(q)}&per_page={limit}"


def github_view_url(repo: str, number: int) -> str:
    return f"https://api.github.com/repos/{repo}/issues/{number}"


def bearer_header(token: str) -> tuple[str, str]:
    return ("Authorization", f"Bearer {token}")
