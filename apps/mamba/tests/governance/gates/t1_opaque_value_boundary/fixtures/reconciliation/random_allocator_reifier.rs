static RANDOMS: u64 = 0;
static RANDOM_IDS: u64 = 0;

fn alloc_random_id() -> MbValue {
    MbValue::from_int(1)
}

fn make_handle() -> u64 {
    let id = alloc_random_id();
    let _ = RANDOMS;
    let _ = RANDOM_IDS;
    id
}

fn publish_handle(id: u64) -> MbValue {
    MbValue::from_int(id as i64)
}
