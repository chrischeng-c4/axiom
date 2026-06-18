// MbValue is a newtype around u64; the JIT passes it by value as a 64-bit word.
#![allow(improper_ctypes_definitions)]

//! FFI functions exposed by `cclab-runtime-mamba` to Mamba scripts.
//!
//! All functions follow the Mamba native-call ABI:
//! ```text
//! extern "C" fn name(args: *const MbValue, nargs: usize) -> MbValue
//! ```
//!
//! # Exposed API
//!
//! | Symbol                | Mamba call                                   |
//! |-----------------------|----------------------------------------------|
//! | `mb_runtime_serve`    | `uvicorn.run(app, host=..., port=...)`       |
//! | `mb_runtime_spawn`    | `asyncio.create_task(coro)`                  |
//! | `mb_runtime_sleep`    | `await asyncio.sleep(seconds)`               |
//! | `mb_runtime_gather`   | `await asyncio.gather(*coros)` (stub)        |

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

use cclab_mamba_registry::convert::mb_wrap_native;
use cclab_mamba_registry::MbValue;

use crate::types::{MbRouter, MbServerHandle, MbTask};

// ── Global Tokio runtime ──────────────────────────────────────────────────────

static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("cclab-runtime-mamba: failed to create Tokio runtime")
});

/// Monotonically increasing task ID counter.
static TASK_COUNTER: AtomicU64 = AtomicU64::new(1);

// ── Helpers ───────────────────────────────────────────────────────────────────

#[inline]
unsafe fn arg(args: *const MbValue, nargs: usize, idx: usize) -> MbValue {
    if idx < nargs {
        unsafe { *args.add(idx) }
    } else {
        MbValue::none()
    }
}

fn read_str(v: MbValue) -> Option<String> {
    cclab_mamba_registry::test_ops::init();
    unsafe { cclab_mamba_registry::rc::read_obj_str(v) }
}

// ── Handler factory ───────────────────────────────────────────────────────────

/// Build a simple Axum handler that returns a fixed 200 OK.
///
/// For this binding prototype, Mamba function pointers cannot yet be called
/// from an async Axum context (requires the Mamba JIT to be linked).  Instead
/// we register a stub handler that returns an empty 200 response so the server
/// starts and routes are reachable.
fn make_stub_handler() -> axum::routing::MethodRouter {
    axum::routing::get(|| async {
        axum::response::Response::builder()
            .status(200)
            .body(axum::body::Body::empty())
            .unwrap()
    })
}

/// Select the Axum method router factory based on an HTTP method string.
fn method_router_for(method: &str) -> axum::routing::MethodRouter {
    match method {
        "GET" => axum::routing::get(|| async {
            axum::response::Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap()
        }),
        "POST" => axum::routing::post(|| async {
            axum::response::Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap()
        }),
        "PUT" => axum::routing::put(|| async {
            axum::response::Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap()
        }),
        "DELETE" => axum::routing::delete(|| async {
            axum::response::Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap()
        }),
        "PATCH" => axum::routing::patch(|| async {
            axum::response::Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap()
        }),
        _ => make_stub_handler(),
    }
}

fn router_from_route_table(mb_router: Option<&MbRouter>) -> axum::Router {
    let mut axum_router = axum::Router::new();

    if let Some(mb_router) = mb_router {
        for route in &mb_router.routes {
            let full_path = if mb_router.prefix.is_empty() {
                route.path.clone()
            } else {
                format!("{}{}", mb_router.prefix, route.path)
            };
            let full_path = if full_path.starts_with('/') {
                full_path
            } else {
                format!("/{full_path}")
            };

            let method = route.method.to_ascii_uppercase();
            axum_router = axum_router.route(&full_path, method_router_for(&method));
        }
    }

    axum_router
}

// ── mb_runtime_serve ──────────────────────────────────────────────────────────

/// Start an HTTP server from an [`MbRouter`] definition.
///
/// # ABI
/// ```text
/// args[0] = router  (MbValue::Ptr → MbRouter)
/// args[1] = host    (MbValue::Ptr → heap String, default "0.0.0.0")
/// args[2] = port    (MbValue::Int, default 8000)
/// ```
///
/// **Blocking**: this function blocks the calling thread until the server is
/// shut down (CTRL-C or signal).  Returns `MbValue::none()`.
///
/// # Note on handler dispatch
///
/// In this prototype, each registered route maps to a stub Axum handler that
/// returns an empty 200 OK.  Full Mamba → Axum function dispatch will be wired
/// once the Mamba JIT exposes a stable cross-thread call interface.
#[no_mangle]
pub unsafe extern "C" fn mb_runtime_serve(args: *const MbValue, nargs: usize) -> MbValue {
    let router_val = unsafe { arg(args, nargs, 0) };
    let host_val = unsafe { arg(args, nargs, 1) };
    let port_val = unsafe { arg(args, nargs, 2) };

    let host = read_str(host_val).unwrap_or_else(|| "0.0.0.0".to_string());
    let port = port_val.as_int().unwrap_or(8000) as u16;

    let router_ptr = router_val.as_ptr().unwrap_or(0);
    let mb_router = if router_ptr == 0 {
        None
    } else {
        Some(unsafe { &*(router_ptr as *const MbRouter) })
    };
    let axum_router = router_from_route_table(mb_router);
    let addr = format!("{host}:{port}");

    TOKIO_RT.block_on(async move {
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                if let Err(e) = axum::serve(listener, axum_router).await {
                    eprintln!("cclab-runtime-mamba: server error: {e}");
                }
            }
            Err(e) => {
                eprintln!("cclab-runtime-mamba: bind error for {addr}: {e}");
            }
        }
    });

    // Return a handle (available for introspection after server exits).
    mb_wrap_native(MbServerHandle {
        host,
        port,
        router_ptr,
    })
}

// ── mb_runtime_spawn ──────────────────────────────────────────────────────────

/// Spawn a Mamba coroutine on the global Tokio runtime.
///
/// # ABI
/// ```text
/// args[0] = coro_ptr  (MbValue::Func — Mamba coroutine pointer)
/// ```
/// Returns an opaque PTR to [`MbTask`].
///
/// # Note
///
/// In this prototype the function pointer is stored for future dispatch.
/// The actual Mamba coroutine execution is not yet wired — the spawned task
/// immediately marks itself done.
#[no_mangle]
pub unsafe extern "C" fn mb_runtime_spawn(args: *const MbValue, nargs: usize) -> MbValue {
    let coro_val = unsafe { arg(args, nargs, 0) };
    let _coro_ptr = coro_val.as_func().unwrap_or(0);

    let task_id = TASK_COUNTER.fetch_add(1, Ordering::Relaxed);
    let task = MbTask::new(task_id);

    // Spawn a no-op task that immediately completes.
    // TODO: wire actual Mamba coroutine dispatch when JIT interface is stable.
    TOKIO_RT.spawn(async move {
        // In a real implementation, call the Mamba function via the JIT.
    });

    let task = mb_wrap_native(task);
    // Mark done immediately since we can't drive the Mamba coroutine yet.
    if let Some(addr) = task.as_ptr() {
        if addr != 0 {
            let t = unsafe { &*(addr as *const MbTask) };
            t.mark_done();
        }
    }
    task
}

// ── mb_runtime_sleep ──────────────────────────────────────────────────────────

/// Block for `seconds`.
///
/// Uses `std::thread::sleep` for the underlying delay since this is always
/// called from an `extern "C"` blocking context.  Tokio's `time::sleep`
/// requires entering an async executor context which is not guaranteed when
/// called from the Mamba JIT.
///
/// # ABI
/// ```text
/// args[0] = seconds  (MbValue::Float or MbValue::Int)
/// ```
/// Returns `MbValue::none()`.
#[no_mangle]
pub unsafe extern "C" fn mb_runtime_sleep(args: *const MbValue, nargs: usize) -> MbValue {
    let secs_val = unsafe { arg(args, nargs, 0) };
    let secs = secs_val
        .as_float()
        .or_else(|| secs_val.as_int().map(|i| i as f64))
        .unwrap_or(0.0)
        .max(0.0);

    std::thread::sleep(Duration::from_secs_f64(secs));
    MbValue::none()
}

// ── mb_runtime_gather ─────────────────────────────────────────────────────────

/// Gather multiple Mamba coroutines (stub).
///
/// # ABI
/// ```text
/// args[0] = coros_list  (MbValue::Ptr → Vec<MbValue> of Func ptrs)
/// ```
/// Returns `MbValue::none()`.
///
/// # Note
///
/// Full implementation requires Mamba coroutine dispatch; this stub accepts
/// the call and returns immediately.
#[no_mangle]
pub unsafe extern "C" fn mb_runtime_gather(_args: *const MbValue, _nargs: usize) -> MbValue {
    // TODO: iterate coros_list, spawn each as a Tokio task, join_all.
    MbValue::none()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MbRouteEntry;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn test_sleep_zero() {
        let args = [MbValue::from_float(0.0)];
        let result = unsafe { mb_runtime_sleep(args.as_ptr(), 1) };
        assert!(result.is_none(), "sleep returns None");
    }

    #[test]
    fn test_sleep_int() {
        let args = [MbValue::from_int(0)];
        let result = unsafe { mb_runtime_sleep(args.as_ptr(), 1) };
        assert!(result.is_none());
    }

    #[test]
    fn test_spawn_returns_task() {
        let fn_ptr = MbValue::from_func(0xCAFE);
        let args = [fn_ptr];
        let task_val = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };
        assert!(task_val.is_ptr(), "spawn returns a task ptr");

        let addr = task_val.as_ptr().unwrap();
        let task = unsafe { &*(addr as *const MbTask) };
        assert!(task.task_id > 0, "task_id should be > 0");
        assert!(task.done(), "stub task should be done immediately");
    }

    #[test]
    fn test_gather_stub() {
        let args: [MbValue; 0] = [];
        let result = unsafe { mb_runtime_gather(args.as_ptr(), 0) };
        assert!(result.is_none(), "gather stub returns None");
    }

    #[test]
    fn test_multiple_spawns_have_distinct_ids() {
        let fn_ptr = MbValue::from_func(0);
        let args = [fn_ptr];

        let t1 = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };
        let t2 = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };

        let id1 = unsafe { &*(t1.as_ptr().unwrap() as *const MbTask) }.task_id;
        let id2 = unsafe { &*(t2.as_ptr().unwrap() as *const MbTask) }.task_id;

        assert_ne!(id1, id2, "each spawn should get a unique task ID");
    }

    #[tokio::test]
    async fn serve_route_table_maps_prefix_paths_and_methods_to_axum_router() {
        let mb_router = MbRouter {
            prefix: "/api".to_string(),
            routes: vec![
                MbRouteEntry {
                    method: "GET".to_string(),
                    path: "/health".to_string(),
                    handler_fn_ptr: 0x1,
                },
                MbRouteEntry {
                    method: "post".to_string(),
                    path: "/items".to_string(),
                    handler_fn_ptr: 0x2,
                },
            ],
        };

        let router = router_from_route_table(Some(&mb_router));

        let health = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::OK);

        let create = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create.status(), StatusCode::OK);

        let missing = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}
