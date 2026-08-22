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

static DECOY_RETURN_TABLE: std::sync::OnceLock<std::collections::HashMap<u64, MbValue>> =
    std::sync::OnceLock::new();

fn decoy_return_put(slot: u64, value: MbValue) {
    let table = DECOY_RETURN_TABLE.get_or_init(|| std::collections::HashMap::new());
    table.insert(slot, value);
}

fn decoy_returned(noise: u64, slot: u64, value: MbValue) -> u64 {
    decoy_return_put(slot, value);
    noise
}

fn decoy_return_api(value: MbValue) -> MbValue {
    let raw = 9;
    let unrelated = decoy_returned(raw, value, value);
    MbValue::from_int(unrelated as i64)
}
