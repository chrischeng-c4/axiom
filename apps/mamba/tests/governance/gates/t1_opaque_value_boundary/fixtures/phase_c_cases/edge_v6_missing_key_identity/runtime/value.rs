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

static SEEDED_FROM_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
thread_local! {
    static SEEDED_FROM_TABLE: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn seeded_from_put(slot: u64, value: MbValue) {
    SEEDED_FROM_TABLE.with(|table| table.borrow_mut().insert(slot + 0, value));
}

fn seeded_from_alloc(value: MbValue) -> u64 {
    let slot = SEEDED_FROM_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    seeded_from_put(slot, value);
    slot
}

fn seeded_from_api(value: MbValue) -> MbValue {
    let raw = seeded_from_alloc(value);
    MbValue::from_int(raw as i64)
}
