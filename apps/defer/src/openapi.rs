// HANDWRITE-BEGIN gap="missing-generator:logic:defer-openapi" tracker="#766" reason="One generated OpenAPI document shared by spec, server, docs, and client codegen."
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Defer HTTP API",
        description = "Raft-backed delayed HTTP push queue over HTTP/1.1 and h2c."
    ),
    paths(
        crate::server::queue_get,
        crate::server::queue_put,
        crate::server::queue_control,
        crate::server::task_create,
        crate::server::task_create_batch,
        crate::server::task_status,
        crate::server::task_cancel,
        crate::server::dispatch_one,
        crate::server::admin_backup,
    ),
    components(schemas(
        crate::Target,
        crate::CreateTask,
        crate::QueuePolicy,
        crate::QueueControlState,
        crate::QueueSnapshot,
        crate::TaskStatus,
        crate::DispatchDisposition,
        crate::DispatchReport,
        crate::server::QueueControlRequest,
        crate::server::CreateTasksRequest,
        crate::server::CreateTasksResponse,
        crate::server::TaskStatusResponse,
        service_http::ErrorEnvelope,
    ))
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
// HANDWRITE-END
