mod integer_handle_registry {
    fn register() {}
    fn retain() {}
}

enum IterKind {
    Range,
}

const HANDLE_MIN_ID: u64 = 1;

fn semantic_alloc(value: MbValue) -> u64 {
    let slot = SEM_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    SEM_TABLE.with(|table| table.borrow_mut().insert(slot, value));
    slot
}

fn semantic_lookup(slot: u64) -> MbValue {
    let raw = SEM_TABLE.with(|table| table.borrow_mut().get(slot));
    raw
}

fn alloc_iter_id() -> u64 {
    SEM_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn alloc_random_id() -> u64 {
    SEM_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

static SEM_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
thread_local! {
    static SEM_TABLE: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static ITERATORS: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
    static RANDOMS: std::cell::RefCell<std::collections::HashMap<u64, MbValue>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

fn route_central_array(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::register();
    MbValue::from_int(slot as i64)
}

fn route_central_queue(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_central_hashlib(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    let _threshold = HANDLE_MIN_ID;
    MbValue::from_int(slot as i64)
}

fn route_central_hmac(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    let _live = [slot].iter().next();
    MbValue::from_int(slot as i64)
}

fn route_central_decimal(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    MbValue::from_int(slot as i64)
}

fn route_central_graphlib(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    let raw = semantic_lookup(slot);
    raw.as_int()
}

fn route_central_json(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_central_uuid(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_central_fractions(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_central_random(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_central_ipaddress(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_direct_iter_store(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    let _iter = alloc_iter_id();
    MbValue::from_int(slot as i64)
}

fn route_direct_range(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    ITERATORS.with(|iterators| iterators.borrow());
    let _range = IterKind::Range;
    let raw = semantic_lookup(slot);
    raw.as_int()
}

fn route_direct_closure(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_direct_generator(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_direct_cell(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_direct_coroutine(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_direct_task(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_direct_file(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    integer_handle_registry::retain();
    MbValue::from_int(slot as i64)
}

fn route_barrier(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    let raw = semantic_lookup(slot);
    let barrier_id = slot;
    let _field = 0;
    raw.as_int()
}

fn route_native_random(value: MbValue) -> MbValue {
    let slot = semantic_alloc(value);
    let _native = alloc_random_id();
    RANDOMS.with(|randoms| randoms.borrow_mut().insert(slot, value));
    MbValue::from_int(slot as i64)
}
