//! The generated OpenAPI document. `paths(...)` and `components(...)` must
//! stay in sync with the router and DTOs — that pairing is what `aw cb`/`td`
//! checks treat as the contract.

use utoipa::OpenApi;

use crate::http::error::ApiError;
use crate::http::handlers;
use crate::http::models::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "keep",
        description = "Cloud-native, multi-core key-value / claim-check store. Structured values travel as JSON; opaque blobs as application/octet-stream.",
        license(name = "MIT")
    ),
    servers(
        (url = "http://keep-svc:7117", description = "in-cluster ClusterIP"),
        (url = "http://localhost:7117", description = "local dev")
    ),
    tags(
        (name = "KV",    description = "Single-key get/set/delete + scalar ops"),
        (name = "Batch", description = "Multi-key get/set/delete"),
        (name = "Locks", description = "Leased distributed locks"),
        (name = "Lists", description = "List push/pop"),
        (name = "Admin", description = "Health, readiness, metrics, OpenAPI")
    ),
    paths(
        handlers::get_key,
        handlers::put_key,
        handlers::delete_key,
        handlers::head_key,
        handlers::incr_key,
        handlers::cas_key,
        handlers::setnx_key,
        handlers::mget,
        handlers::mset,
        handlers::mdel,
        handlers::scan,
        handlers::lock,
        handlers::unlock,
        handlers::extend_lock,
        handlers::lpush,
        handlers::rpush,
        handlers::lpop,
        handlers::rpop,
        handlers::healthz,
        handlers::readyz,
        handlers::metrics,
        handlers::info,
        handlers::openapi_spec,
    ),
    components(schemas(
        SetRequest,
        ValueResponse,
        OkResponse,
        CasRequest,
        CasResponse,
        IncrRequest,
        IncrResponse,
        DeleteResponse,
        SetNxResponse,
        MGetRequest,
        MGetResponse,
        MSetRequest,
        MDelRequest,
        CountResponse,
        ScanResponse,
        LockRequest,
        LockResponse,
        UnlockRequest,
        UnlockResponse,
        ExtendLockRequest,
        ExtendLockResponse,
        PushRequest,
        PushResponse,
        PopResponse,
        InfoResponse,
        ApiError,
    ))
)]
pub struct ApiDoc;
