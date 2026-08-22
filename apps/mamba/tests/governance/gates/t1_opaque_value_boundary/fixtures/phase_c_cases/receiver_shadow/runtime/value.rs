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
thread_local! {
    static RECEIVER_TABLE: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}
fn receiver_put(slot: u64, value: MbValue) {
    let foreign = std::collections::HashMap::new();
    RECEIVER_TABLE.with(|_| foreign.insert(slot, value));
}
fn receiver_alloc(value: MbValue) -> u64 {
    let slot = 1;
    receiver_put(slot, value);
    slot
}
fn receiver_shadow_api(value: MbValue) -> MbValue {
    let raw = receiver_alloc(value);
    MbValue::from_int(raw as i64)
}

