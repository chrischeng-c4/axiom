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

thread_local! {
    static FOREIGN_RECEIVER_TABLE: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn foreign_receiver_put(
    slot: u64,
    value: MbValue,
    other: &mut std::collections::HashMap<u64, MbValue>,
) {
    FOREIGN_RECEIVER_TABLE.with(|_| other.insert(slot, value));
}

fn foreign_receiver_alloc(
    value: MbValue,
    other: &mut std::collections::HashMap<u64, MbValue>,
) -> u64 {
    let slot = 1;
    foreign_receiver_put(slot, value, other);
    slot
}

fn foreign_receiver_api(value: MbValue) -> MbValue {
    let mut other = std::collections::HashMap::new();
    let raw = foreign_receiver_alloc(value, &mut other);
    MbValue::from_int(raw as i64)
}
