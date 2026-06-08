---
number: 964
title: "epic(ecosystem): Align cclab.* Python API to ecosystem conventions"
state: open
labels: [P0, type:tracking]
---

# #964 — epic(ecosystem): Align cclab.* Python API to ecosystem conventions

## Vision

cclab.* claims to replace Python ecosystem (Pydantic, FastAPI, SQLAlchemy, etc.), but the current Python API doesn't match conventions. Fix from Rust level, not aliases.

First consumer: Conductor.

## Per-Crate Issues

### cclab-schema — needs BaseModel + Field
- Currently: only exports BaseSettings
- BaseModel is in cclab-api (wrong place)
- Need: `from cclab.schema import BaseModel, Field, validator`
- Move BaseModel from api to schema crate

### cclab-pg — needs declarative ORM
- Currently: Table + Column + QueryBuilder (low-level)
- Need: declarative base pattern like SQLAlchemy
- `from cclab.pg import Base, Mapped, mapped_column, relationship`
- Or design new API, but must support: model definition, relationships, async session

### cclab-api — needs Router
- Currently: App + Depends + HTTPException
- Need: `from cclab.api import Router` (sub-routers like FastAPI APIRouter)
- Confirm: response types, middleware

### cclab-qc — needs mark namespace + raises
- Currently: test, fixture, parametrize
- Need: `mark.asyncio`, `mark.parametrize`, `raises(Exception)`
- pytest-compatible surface

### cclab-fetch — needs clear HttpClient
- Currently: PyHttpClient in PyO3
- Need: clean `from cclab.fetch import HttpClient` with async get/post/put/delete

## Approach

For each crate:
1. Define target Python API (what Conductor needs)
2. Update Rust crate if needed
3. Update PyO3 bindings
4. Update Python __init__.py
5. Test with Conductor
