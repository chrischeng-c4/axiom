static DBM_STORES: HashMap<u64, Store> = HashMap::new();
static NEXT_LZMA_ID: AtomicU64 = AtomicU64::new(0);
static CONNS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static NEXT_VAR_ID: AtomicU64 = AtomicU64::new(0);
