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
use std::cell::RefCell;
use std::collections::HashMap;
static FOREIGN_TABLE: RefCell<HashMap<u64, MbValue>> = RefCell::new(HashMap::new());
static FOREIGN_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn foreign_alloc(value: MbValue) -> u64 {
    let slot = FOREIGN_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    foreign_put(slot, value);
    slot
}
fn foreign_put(slot: u64, value: MbValue) {
    FOREIGN_TABLE.with(|table| table.insert(slot, value));
}
fn foreign_api(value: MbValue) -> MbValue {
    let raw = foreign_alloc(value);
    MbValue::from_int(raw as i64)
}
fn foreign_put_shadow(slot: u64, value: MbValue) {
    FOREIGN_TABLE.with(|table| foreign_receiver.insert(slot, value));
}

