fn route_central_array(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
        MbValue::from_int(slot as i64);
    MbValue::from_int(slot as i64)
}

fn semantic_alloc(value: MbValue) -> u64 {
    let slot = SEM_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    SEM_TABLE.with(|table| table.borrow_mut().insert(slot, value));
    SEM_TABLE.with(|table| table.borrow_mut().insert(slot, value));
    slot
}

static SEM_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
thread_local! {
    static SEM_TABLE: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

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


fn multi_path_alloc(value: MbValue) -> u64 {
    let slot = SEM_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let _ = value;
    slot
}

fn unseeded_multi_path_zero_terminal(value: MbValue) -> MbValue {
    let slot = multi_path_alloc(value);
    SEM_TABLE.with(|table| table.borrow_mut().insert(99, value));
    let _first = MbValue::from_int(slot as i64);
    let _second = MbValue::from_int(slot as i64);
    value
}

