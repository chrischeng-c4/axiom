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

static RENAMED_AS_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
thread_local! {
    static RENAMED_AS_TABLE: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn renamed_as_put(slot: u64, value: MbValue) {
    RENAMED_AS_TABLE.with(|table| table.borrow_mut().insert(slot, value));
}

fn renamed_as_alloc(value: MbValue) -> u64 {
    let slot = RENAMED_AS_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    renamed_as_put(slot, value);
    slot
}

fn renamed_as_lookup(slot: u64) -> MbValue {
    let value = RENAMED_AS_TABLE.with(|table| table.borrow().get(slot));
    value
}

fn renamed_as_api(value: MbValue) -> i64 {
    let slot = renamed_as_alloc(value);
    let looked = renamed_as_lookup(slot);
    looked.as_int()
}
