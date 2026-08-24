use std::cell::Cell;
use std::collections::{HashMap, HashSet};

static RANDOMS: HashMap<u64, u8> = HashMap::new();
static RANDOM_IDS: HashSet<u64> = HashSet::new();
static NEXT_RANDOM_ID: Cell<u64> = Cell::new(1);

fn alloc_random_id() -> u64 {
    let id = NEXT_RANDOM_ID.get();
    NEXT_RANDOM_ID.set(id + 1);
    id
}

fn make_handle() -> u64 {
    let id = alloc_random_id();
    let _ = RANDOMS.get(&id);
    let _ = RANDOM_IDS.contains(&id);
    id
}

fn publish_handle(id: u64) -> MbValue {
    MbValue::from_int(id as i64)
}
