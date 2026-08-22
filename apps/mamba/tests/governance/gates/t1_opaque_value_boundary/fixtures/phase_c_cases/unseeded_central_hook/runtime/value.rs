static UNSEEDED_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
thread_local! {
    static UNSEEDED_HOOKS: std::cell::RefCell<Vec<u64>> = std::cell::RefCell::new(Vec::new());
}

fn route_central_array() {}
fn route_central_queue() {}
fn route_central_hashlib() {}
fn route_central_hmac() {}
fn route_central_decimal() {}
fn route_central_graphlib() {}
fn route_central_json() {}
fn route_central_uuid() {}
fn route_central_fractions() {}
fn route_central_random() {}
fn route_central_ipaddress() {}
fn route_direct_iter_store() {}
fn route_direct_range() {}
fn route_direct_closure() {}
fn route_direct_generator() {}
fn route_direct_cell() {}
fn route_direct_coroutine() {}
fn route_direct_task() {}
fn route_direct_file() {}
fn route_barrier() {}
fn route_native_random() {}
fn unseeded_central() {
    let id = UNSEEDED_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    UNSEEDED_HOOKS.with(|hooks| hooks.borrow_mut().push(id));
    integer_handle_registry::register();
    MbValue::from_int(id as i64)
}
