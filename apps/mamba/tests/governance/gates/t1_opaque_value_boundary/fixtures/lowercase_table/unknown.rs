static dbm_stores: HashMap<u64, Store> = HashMap::new();
static next_lzma_id: AtomicU64 = AtomicU64::new(0);
