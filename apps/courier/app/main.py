import os
import re
from enum import Enum
from typing import List, Optional
import httpx
from fastapi import FastAPI, HTTPException, Query, status
from pydantic import BaseModel, Field

app = FastAPI(title="courier", description="stateless GitHub issues proxy")

GITHUB_TOKEN = os.environ.get("COURIER_GITHUB_TOKEN", "").strip()
ALLOWED_REPOS = [
    r.strip() for r in os.environ.get("COURIER_ALLOWED_REPOS", "chrischeng-c4/axiom").split(",") if r.strip()
]

# Fail fast at startup if no token is configured
if not GITHUB_TOKEN:
    raise RuntimeError("COURIER_GITHUB_TOKEN must be set")

class OpType(str, Enum):
    CREATE = "create"
    COMMENT = "comment"
    UPDATE = "update"

class IssueOp(BaseModel):
    op: OpType
    title: Optional[str] = None
    body: Optional[str] = None
    labels: Optional[List[str]] = None
    number: Optional[int] = None
    state: Optional[str] = None

class BatchMutationRequest(BaseModel):
    repo: str
    ops: List[IssueOp] = Field(..., max_length=10)

def is_repo_allowed(repo: str) -> bool:
    return repo in ALLOWED_REPOS

def parse_repo(repo: str) -> tuple[str, str]:
    parts = repo.split("/", 1)
    if len(parts) != 2:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid repo format. Must be owner/name"
        )
    return parts[0], parts[1]

async def forward_to_github(
    client: httpx.AsyncClient,
    method: str,
    path: str,
    json_data: Optional[dict] = None,
    params: Optional[dict] = None
) -> httpx.Response:
    url = f"https://api.github.com{path}"
    headers = {
        "Accept": "application/vnd.github+json",
        "Authorization": f"Bearer {GITHUB_TOKEN}",
        "User-Agent": "courier/python"
    }
    resp = await client.request(method, url, headers=headers, json=json_data, params=params)
    return resp

@app.get("/healthz")
@app.get("/readyz")
def health_check():
    return {"status": "ok"}

@app.get("/issues")
async def get_issues(
    repo: str = Query(..., description="GitHub repo owner/name"),
    number: Optional[int] = Query(None, description="Issue number for single view"),
    state: str = Query("open", description="Issue state (open, closed, all)"),
    q: Optional[str] = Query(None, description="Search text query"),
    limit: int = Query(20, description="Max results limit")
):
    if not is_repo_allowed(repo):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"{repo} is not in COURIER_ALLOWED_REPOS"
        )

    owner, name = parse_repo(repo)

    async with httpx.AsyncClient() as client:
        if number is not None:
            # Single view
            resp = await forward_to_github(client, "GET", f"/repos/{owner}/{name}/issues/{number}")
            if resp.status_code != 200:
                raise HTTPException(status_code=resp.status_code, detail=resp.text)
            return resp.json()
        else:
            # Search/List
            query = f"repo:{owner}/{name} is:issue"
            if state != "all":
                query += f" state:{state}"
            if q:
                query += f" {q}"
            
            params = {
                "q": query,
                "per_page": limit
            }
            resp = await forward_to_github(client, "GET", "/search/issues", params=params)
            if resp.status_code != 200:
                raise HTTPException(status_code=resp.status_code, detail=resp.text)
            return resp.json()

@app.post("/issues")
async def mutate_issues(request: BatchMutationRequest):
    if not is_repo_allowed(request.repo):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"{request.repo} is not in COURIER_ALLOWED_REPOS"
        )

    owner, name = parse_repo(request.repo)
    results = []

    async with httpx.AsyncClient() as client:
        for op in request.ops:
            try:
                if op.op == OpType.CREATE:
                    payload = {"title": op.title}
                    if op.body is not None:
                        payload["body"] = op.body
                    if op.labels is not None:
                        payload["labels"] = op.labels
                    
                    resp = await forward_to_github(client, "POST", f"/repos/{owner}/{name}/issues", json_data=payload)
                    results.append({"status": resp.status_code, "body": resp.json() if resp.status_code < 400 else resp.text})

                elif op.op == OpType.COMMENT:
                    if op.number is None:
                        results.append({"status": 400, "error": "missing issue number for comment"})
                        continue
                    # Reopen issue first (idempotent, match Rust behavior)
                    await forward_to_github(client, "PATCH", f"/repos/{owner}/{name}/issues/{op.number}", json_data={"state": "open"})
                    
                    payload = {"body": op.body}
                    resp = await forward_to_github(client, "POST", f"/repos/{owner}/{name}/issues/{op.number}/comments", json_data=payload)
                    results.append({"status": resp.status_code, "body": resp.json() if resp.status_code < 400 else resp.text})

                elif op.op == OpType.UPDATE:
                    if op.number is None:
                        results.append({"status": 400, "error": "missing issue number for update"})
                        continue
                    payload = {}
                    if op.state is not None:
                        payload["state"] = op.state
                    
                    resp = await forward_to_github(client, "PATCH", f"/repos/{owner}/{name}/issues/{op.number}", json_data=payload)
                    results.append({"status": resp.status_code, "body": resp.json() if resp.status_code < 400 else resp.text})

            except Exception as e:
                results.append({"status": 500, "error": "internal_error", "message": str(e)})

    return results
