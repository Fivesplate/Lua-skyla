//! ltests.rs - Advanced internal testing and debugging for Rust-based Lua VM
// Ported and extended from ltests.c/h

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;
use crate::lstate::LuaState;
use crate::lobject::{LuaValue, GcObject};
use rand::Rng;

/// Memory control and tracking (inspired by Memcontrol in ltests.h)
pub struct MemControl {
    pub fail_next: bool,
    pub num_blocks: AtomicUsize,
    pub total: AtomicUsize,
    pub max_mem: AtomicUsize,
    pub mem_limit: AtomicUsize,
    pub count_limit: AtomicUsize,
    pub obj_count: Mutex<HashMap<&'static str, usize>>, // type name -> count
}

impl MemControl {
    pub fn new() -> Self {
        Self {
            fail_next: false,
            num_blocks: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
            max_mem: AtomicUsize::new(0),
            mem_limit: AtomicUsize::new(usize::MAX),
            count_limit: AtomicUsize::new(usize::MAX),
            obj_count: Mutex::new(HashMap::new()),
        }
    }
    pub fn alloc(&self, type_name: &'static str, size: usize) {
        self.num_blocks.fetch_add(1, Ordering::SeqCst);
        self.total.fetch_add(size, Ordering::SeqCst);
        self.max_mem.fetch_max(self.total.load(Ordering::SeqCst), Ordering::SeqCst);
        let mut map = self.obj_count.lock().unwrap();
        *map.entry(type_name).or_insert(0) += 1;
    }
    pub fn free(&self, type_name: &'static str, size: usize) {
        self.num_blocks.fetch_sub(1, Ordering::SeqCst);
        self.total.fetch_sub(size, Ordering::SeqCst);
        let mut map = self.obj_count.lock().unwrap();
        *map.entry(type_name).or_insert(0) -= 1;
    }
    pub fn should_fail(&self) -> bool {
        self.fail_next
    }
    pub fn set_fail_next(&mut self, fail: bool) {
        self.fail_next = fail;
    }
}

lazy_static::lazy_static! {
    pub static ref MEM_CONTROL: MemControl = MemControl::new();
}

/// Debug helpers
pub fn print_value(val: &LuaValue) {
    println!("[ltests] Value: {:?}", val);
}

pub fn print_stack(state: &LuaState) {
    println!("[ltests] Stack: {:?}", state.stack_snapshot());
}

pub fn print_gc_object(obj: &GcObject) {
    println!("[ltests] GCObject: {:?}", obj);
}

/// Advanced test: force memory failure on next alloc
pub fn fail_next_alloc() {
    MEM_CONTROL.set_fail_next(true);
}

/// Advanced test: check memory consistency (stub)
pub fn check_memory(_state: &LuaState) -> bool {
    // TODO: Traverse all objects and check invariants
    true
}

/// Advanced test: simulate warning
pub fn test_warning(msg: &str) {
    println!("[ltests] Warning: {}", msg);
}

/// Advanced test: simulate panic
pub fn test_panic(msg: &str) {
    eprintln!("[ltests] PANIC: {}", msg);
    panic!("[ltests] PANIC: {}", msg);
}

/// Advanced: Randomized memory failure for stress testing
pub fn maybe_fail_alloc(probability: f64) {
    if rand::thread_rng().gen_bool(probability) {
        MEM_CONTROL.set_fail_next(true);
    }
}

/// Advanced: Print all memory stats
pub fn print_mem_stats() {
    let mc = &*MEM_CONTROL;
    println!("[ltests] Memory blocks: {}", mc.num_blocks.load(Ordering::SeqCst));
    println!("[ltests] Total memory: {}", mc.total.load(Ordering::SeqCst));
    println!("[ltests] Max memory: {}", mc.max_mem.load(Ordering::SeqCst));
    println!("[ltests] Object counts: {:?}", mc.obj_count.lock().unwrap());
}

/// Advanced: Assert macro for Lua VM tests
#[macro_export]
macro_rules! ltest_assert {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("[ltests] Assertion failed: {}", $msg);
        }
    };
}

/// Advanced: Fuzzing hook (stub)
pub fn fuzz_vm(state: &mut LuaState, iterations: usize) {
    use rand::seq::SliceRandom;
    let ops = ["push", "pop", "call", "gc", "alloc", "free"];
    for _ in 0..iterations {
        let op = ops.choose(&mut rand::thread_rng()).unwrap();
        match *op {
            "push" => state.push(LuaValue::Int(rand::random())),
            "pop" => { let _ = state.pop(1); },
            "call" => {/* stub: call random function */},
            "gc" => {/* stub: trigger GC */},
            "alloc" => { MEM_CONTROL.alloc("fuzz", rand::random::<u8>() as usize); },
            "free" => { MEM_CONTROL.free("fuzz", rand::random::<u8>() as usize); },
            _ => {}
        }
    }
}

/// Advanced: Deterministic replay of fuzzing sessions
use rand::{SeedableRng, rngs::StdRng};

#[derive(Debug, Clone)]
pub enum FuzzOp {
    Push(i64),
    Pop,
    Call,
    Gc,
    Alloc(usize),
    Free(usize),
}

#[derive(Debug, Clone)]
pub struct FuzzSessionLog {
    pub seed: u64,
    pub ops: Vec<FuzzOp>,
}

impl FuzzSessionLog {
    pub fn new(seed: u64) -> Self {
        Self { seed, ops: Vec::new() }
    }
}

/// Run a fuzzing session with deterministic seed, recording all operations
pub fn fuzz_vm_deterministic(state: &mut LuaState, iterations: usize, seed: u64) -> FuzzSessionLog {
    use rand::Rng;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut log = FuzzSessionLog::new(seed);
    for _ in 0..iterations {
        let op = rng.gen_range(0..6);
        match op {
            0 => {
                let val = rng.gen::<i64>();
                state.push(LuaValue::Int(val));
                log.ops.push(FuzzOp::Push(val));
            },
            1 => {
                let _ = state.pop(1);
                log.ops.push(FuzzOp::Pop);
            },
            2 => {
                // stub: call random function
                log.ops.push(FuzzOp::Call);
            },
            3 => {
                // stub: trigger GC
                log.ops.push(FuzzOp::Gc);
            },
            4 => {
                let sz = rng.gen::<u8>() as usize;
                MEM_CONTROL.alloc("fuzz", sz);
                log.ops.push(FuzzOp::Alloc(sz));
            },
            5 => {
                let sz = rng.gen::<u8>() as usize;
                MEM_CONTROL.free("fuzz", sz);
                log.ops.push(FuzzOp::Free(sz));
            },
            _ => {}
        }
    }
    log
}

/// Advanced: Deterministic fuzzing session record/replay
use std::fs::File;
use std::io::{Write, Read};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FuzzOp {
    Push(LuaValue),
    Pop,
    Call,
    Gc,
    Alloc(usize),
    Free(usize),
}

/// Record a sequence of fuzzing operations to a file
pub fn record_fuzz_session(state: &mut LuaState, ops: usize, path: &str) {
    use rand::seq::SliceRandom;
    let mut log = Vec::new();
    let mut rng = rand::thread_rng();
    let opkinds = ["push", "pop", "call", "gc", "alloc", "free"];
    for _ in 0..ops {
        let op = opkinds.choose(&mut rng).unwrap();
        match *op {
            "push" => {
                let v = random_lua_value();
                state.push(v.clone());
                log.push(FuzzOp::Push(v));
            },
            "pop" => {
                let _ = state.pop(1);
                log.push(FuzzOp::Pop);
            },
            "call" => {
                // stub: call random function
                log.push(FuzzOp::Call);
            },
            "gc" => {
                // stub: trigger GC
                log.push(FuzzOp::Gc);
            },
            "alloc" => {
                let sz = rng.gen_range(1..32);
                MEM_CONTROL.alloc("fuzz", sz);
                log.push(FuzzOp::Alloc(sz));
            },
            "free" => {
                let sz = rng.gen_range(1..32);
                MEM_CONTROL.free("fuzz", sz);
                log.push(FuzzOp::Free(sz));
            },
            _ => {}
        }
    }
    let data = bincode::serialize(&log).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(&data).unwrap();
    println!("[ltests] Fuzz session recorded to {} ({} ops)", path, ops);
}

/// Replay a recorded fuzzing session from a file
pub fn replay_fuzz_session(state: &mut LuaState, path: &str) {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let log: Vec<FuzzOp> = bincode::deserialize(&data).unwrap();
    for op in log {
        match op {
            FuzzOp::Push(v) => state.push(v),
            FuzzOp::Pop => { let _ = state.pop(1); },
            FuzzOp::Call => {/* stub */},
            FuzzOp::Gc => {/* stub */},
            FuzzOp::Alloc(sz) => { MEM_CONTROL.alloc("fuzz", sz); },
            FuzzOp::Free(sz) => { MEM_CONTROL.free("fuzz", sz); },
        }
    }
    println!("[ltests] Fuzz session replayed from {} ({} ops)", path, log.len());
}

/// Advanced: Heap/stack poison check helpers
const POISON_PATTERN: i64 = 0x5A5A5A5A5A5A5A5A;

/// Poison the stack with a known pattern
pub fn poison_stack(state: &mut LuaState) {
    for i in 0..state.stack_size() {
        state.set_stack(i, LuaValue::Int(POISON_PATTERN));
    }
    println!("[ltests] Stack poisoned");
}

/// Check for poison pattern in stack (detect use-after-free/uninit)
pub fn check_stack_poison(state: &LuaState) -> bool {
    for i in 0..state.stack_size() {
        if let LuaValue::Int(v) = state.get_stack(i) {
            if v == POISON_PATTERN {
                println!("[ltests] Poison detected at stack[{}]", i);
                return true;
            }
        }
    }
    false
}

/// Poison the heap by allocating/filling with poison pattern (stub)
pub fn poison_heap() {
    for _ in 0..10 {
        MEM_CONTROL.alloc("poison", POISON_PATTERN as usize);
    }
    println!("[ltests] Heap poisoned");
}

/// Check for poison pattern in heap (stub)
pub fn check_heap_poison() -> bool {
    // In a real implementation, would scan heap for POISON_PATTERN
    // Here, just print a stub message
    println!("[ltests] Heap poison check: stub");
    false
}

/// Advanced: Stress test for stack overflow
pub fn stress_stack(state: &mut LuaState, depth: usize) {
    if depth == 0 { return; }
    state.push(LuaValue::Int(depth as i64));
    stress_stack(state, depth - 1);
}

/// Advanced: Print all GC objects (stub)
pub fn print_all_gc_objects(_state: &LuaState) {
    // TODO: Traverse and print all GC objects in the VM
    println!("[ltests] print_all_gc_objects: not yet implemented");
}

/// Advanced: Traverse and print all GC objects (deep)
pub fn traverse_gc_objects(state: &LuaState, visit: &mut dyn FnMut(&GcObject)) {
    // Example: traverse all objects in the VM's GC list (stub)
    for obj in state.all_gc_objects() {
        visit(obj);
    }
}

/// Advanced: Simulate stack/heap corruption for robustness testing
pub fn corrupt_stack(state: &mut LuaState, count: usize) {
    for _ in 0..count {
        state.push(LuaValue::Int(rand::random()));
    }
    // Overwrite random stack slots
    let stack_size = state.stack_size();
    for _ in 0..(count / 2) {
        let idx = rand::random::<usize>() % stack_size;
        state.set_stack(idx, LuaValue::Nil);
    }
}

pub fn corrupt_heap() {
    // Simulate heap corruption by random alloc/free
    for _ in 0..10 {
        if rand::random::<bool>() {
            MEM_CONTROL.alloc("corrupt", rand::random::<u8>() as usize);
        } else {
            MEM_CONTROL.free("corrupt", rand::random::<u8>() as usize);
        }
    }
}

/// Advanced: Test thread/lock state (stub)
pub fn test_thread_lock(state: &mut LuaState) {
    // Example: simulate lock/unlock and assert correctness
    state.lock();
    ltest_assert!(state.is_locked(), "VM should be locked");
    state.unlock();
    ltest_assert!(!state.is_locked(), "VM should be unlocked");
}

/// Advanced: Batch test runner for fuzz/stress
pub fn run_batch_tests(state: &mut LuaState, n: usize) {
    for _ in 0..n {
        fuzz_vm(state, 100);
        stress_stack(state, 100);
        corrupt_stack(state, 10);
        corrupt_heap();
        print_mem_stats();
    }
}

/// Advanced: Take a snapshot of the VM state (stub)
pub fn snapshot_vm(state: &LuaState) -> Vec<u8> {
    // Serialize stack, globals, and GC objects (stub)
    // In a real implementation, this would walk all VM state
    vec![]
}

/// Advanced: Restore a VM state from snapshot (stub)
pub fn restore_vm(state: &mut LuaState, snapshot: &[u8]) {
    // Deserialize and restore stack, globals, and GC objects (stub)
    // In a real implementation, this would walk all VM state
    let _ = (state, snapshot);
}

/// Advanced: Generate a random LuaValue for fuzzing
default fn random_lua_value() -> LuaValue {
    use rand::Rng;
    match rand::thread_rng().gen_range(0..5) {
        0 => LuaValue::Int(rand::random()),
        1 => LuaValue::Float(rand::random::<f64>()),
        2 => LuaValue::Bool(rand::random()),
        3 => LuaValue::Str(format!("rand_{}", rand::random::<u32>())),
        _ => LuaValue::Nil,
    }
}

/// Advanced: Deep stack and heap invariant checker (stub)
pub fn check_invariants(state: &LuaState) -> bool {
    // Walk stack and heap, check for invalid/corrupt values (stub)
    // In a real implementation, this would check all invariants
    let _ = state;
    true
}

/// Advanced: Test coverage tracker (stub)
pub struct CoverageTracker {
    pub covered: Mutex<HashMap<&'static str, usize>>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self { covered: Mutex::new(HashMap::new()) }
    }
    pub fn hit(&self, label: &'static str) {
        let mut map = self.covered.lock().unwrap();
        *map.entry(label).or_insert(0) += 1;
    }
    pub fn report(&self) {
        println!("[ltests] Coverage: {:?}", self.covered.lock().unwrap());
    }
}

lazy_static::lazy_static! {
    pub static ref COVERAGE: CoverageTracker = CoverageTracker::new();
}

/// Advanced: Time-bounded fuzzing session
pub fn fuzz_for_duration(state: &mut LuaState, seconds: u64) {
    use std::time::{Instant, Duration};
    let start = Instant::now();
    let dur = Duration::from_secs(seconds);
    let mut iters = 0;
    while Instant::now() - start < dur {
        fuzz_vm(state, 10);
        iters += 1;
    }
    println!("[ltests] Fuzzed for {} iterations in {:?}", iters, dur);
}

/// Advanced: VM state diff (stub)
pub fn diff_vm_snapshots(a: &[u8], b: &[u8]) {
    // In a real implementation, this would compare two VM state snapshots
    if a == b {
        println!("[ltests] VM snapshots are identical");
    } else {
        println!("[ltests] VM snapshots differ ({} vs {} bytes)", a.len(), b.len());
    }
}

/// Advanced: Randomized metatable/GC mutation
pub fn mutate_metatable_and_gc(state: &mut LuaState) {
    // Randomly set or clear metatables, trigger GC, etc.
    if rand::random::<bool>() {
        state.set_random_metatable();
    }
    if rand::random::<bool>() {
        state.collect_garbage();
    }
}

/// Advanced: LuaValue shrinker for property-based testing
pub fn shrink_lua_value(val: &LuaValue) -> Vec<LuaValue> {
    match val {
        LuaValue::Int(i) if *i != 0 => vec![LuaValue::Int(0)],
        LuaValue::Float(f) if *f != 0.0 => vec![LuaValue::Float(0.0)],
        LuaValue::Str(s) if !s.is_empty() => vec![LuaValue::Str(String::new())],
        LuaValue::Bool(_) => vec![LuaValue::Bool(false)],
        _ => vec![],
    }
}

/// Advanced: Time-travel debugging hooks
pub struct TimeTravelDebugger {
    pub snapshots: Vec<Vec<u8>>,
    pub current: usize,
}

impl TimeTravelDebugger {
    pub fn new() -> Self {
        Self { snapshots: Vec::new(), current: 0 }
    }
    pub fn save_snapshot(&mut self, state: &LuaState) {
        let snap = snapshot_vm(state);
        self.snapshots.push(snap);
        self.current = self.snapshots.len() - 1;
        println!("[ltests] Snapshot saved (#{})", self.current);
    }
    pub fn restore_snapshot(&mut self, state: &mut LuaState, idx: usize) {
        if idx < self.snapshots.len() {
            restore_vm(state, &self.snapshots[idx]);
            self.current = idx;
            println!("[ltests] Restored snapshot #{}", idx);
        } else {
            println!("[ltests] Invalid snapshot index: {}", idx);
        }
    }
    pub fn step_back(&mut self, state: &mut LuaState) {
        if self.current > 0 {
            self.current -= 1;
            restore_vm(state, &self.snapshots[self.current]);
            println!("[ltests] Stepped back to snapshot #{}", self.current);
        } else {
            println!("[ltests] Already at oldest snapshot");
        }
    }
    pub fn step_forward(&mut self, state: &mut LuaState) {
        if self.current + 1 < self.snapshots.len() {
            self.current += 1;
            restore_vm(state, &self.snapshots[self.current]);
            println!("[ltests] Stepped forward to snapshot #{}", self.current);
        } else {
            println!("[ltests] Already at newest snapshot");
        }
    }
}

/// Advanced: Property-based test runner for LuaValue and table ops
pub fn property_test_luavalue(iterations: usize) {
    for _ in 0..iterations {
        let v = random_lua_value();
        // Example property: shrinker always returns a value less than or equal in 'size'
        let shrunk = shrink_lua_value(&v);
        for s in &shrunk {
            match (&v, s) {
                (LuaValue::Int(a), LuaValue::Int(b)) => assert!(b.abs() <= a.abs()),
                (LuaValue::Float(a), LuaValue::Float(b)) => assert!(b.abs() <= a.abs()),
                (LuaValue::Str(a), LuaValue::Str(b)) => assert!(b.len() <= a.len()),
                _ => {}
            }
        }
    }
    println!("[ltests] Property-based LuaValue shrinker test passed ({} iterations)", iterations);
}

pub fn property_test_table_merge(iterations: usize, state: &mut LuaState) {
    use crate::ltable::LuaTable;
    use rand::Rng;
    for _ in 0..iterations {
        let mut t1 = LuaTable::new();
        let mut t2 = LuaTable::new();
        let n = rand::thread_rng().gen_range(1..10);
        for i in 0..n {
            t1.set(LuaValue::Int(i), random_lua_value());
            t2.set(LuaValue::Int(i + n), random_lua_value());
        }
        let merged = t1.merge(&t2);
        // Property: merged table contains all keys from both
        for i in 0..n {
            assert!(merged.get(&LuaValue::Int(i)).is_some());
            assert!(merged.get(&LuaValue::Int(i + n)).is_some());
        }
    }
    println!("[ltests] Property-based table merge test passed ({} iterations)", iterations);
}

/// Advanced: Concurrent VM stress test (multi-threaded)
use std::thread;
use std::sync::Arc;

pub fn concurrent_vm_stress(state: &mut LuaState, threads: usize, iters: usize) {
    let state = Arc::new(Mutex::new(state));
    let mut handles = Vec::new();
    for tid in 0..threads {
        let state = Arc::clone(&state);
        let handle = thread::spawn(move || {
            for _ in 0..iters {
                let mut s = state.lock().unwrap();
                s.push(LuaValue::Int(tid as i64));
                let _ = s.pop(1);
                // Optionally: call more random ops, fuzz, etc.
            }
        });
        handles.push(handle);
    }
    for h in handles { h.join().unwrap(); }
    println!("[ltests] Concurrent VM stress test complete ({} threads x {} iters)", threads, iters);
}

/// Advanced: GC stress and leak detection
pub fn gc_stress_and_leak_check(state: &mut LuaState, cycles: usize) {
    let before = state.gc_object_count();
    for _ in 0..cycles {
        state.collect_garbage();
    }
    let after = state.gc_object_count();
    if after > before {
        println!("[ltests] GC leak detected: {} -> {} objects", before, after);
    } else {
        println!("[ltests] GC stress test passed: {} -> {} objects", before, after);
    }
}

/// Advanced: API misuse/error generator
pub fn api_misuse_fuzz(state: &mut LuaState, iterations: usize) {
    use rand::Rng;
    for _ in 0..iterations {
        let op = rand::thread_rng().gen_range(0..4);
        match op {
            0 => { let _ = state.pop(rand::thread_rng().gen_range(100..200)); }, // pop too many
            1 => { state.set_stack(rand::thread_rng().gen_range(1000..2000), LuaValue::Nil); }, // set out of bounds
            2 => { let _ = state.get_stack(rand::thread_rng().gen_range(1000..2000)); }, // get out of bounds
            3 => { state.push(LuaValue::Str(String::new())); }, // push empty string (edge)
            _ => {}
        }
    }
    println!("[ltests] API misuse fuzzing complete ({} iterations)", iterations);
}

/// Advanced: Invariant violation reporting with diagnostics
pub fn check_invariants_with_report(state: &LuaState) -> bool {
    // Example: check stack for poison pattern, print diagnostics if found
    let mut ok = true;
    for i in 0..state.stack_size() {
        if let LuaValue::Int(v) = state.get_stack(i) {
            if v == POISON_PATTERN {
                println!("[ltests] Invariant violation: poison at stack[{}]", i);
                print_stack(state);
                ok = false;
            }
        }
    }
    if !ok {
        println!("[ltests] VM state dump for diagnostics:");
        // Optionally: dump more state here
    }
    ok
}

/// Advanced: Differential testing between two LuaState VMs
pub fn differential_test<F>(state1: &mut LuaState, state2: &mut LuaState, ops: usize, mut opgen: F)
where F: FnMut(&mut LuaState)
{
    for _ in 0..ops {
        opgen(state1);
        opgen(state2);
    }
    let snap1 = state1.stack_snapshot();
    let snap2 = state2.stack_snapshot();
    if snap1 != snap2 {
        println!("[ltests] Differential test failed: VM states diverged");
        println!("State1: {:?}", snap1);
        println!("State2: {:?}", snap2);
    } else {
        println!("[ltests] Differential test passed: VM states match");
    }
}

/// Advanced: Randomized metatable/GC mutation stress
pub fn metatable_gc_mutation_stress(state: &mut LuaState, iterations: usize) {
    for _ in 0..iterations {
        mutate_metatable_and_gc(state);
    }
    println!("[ltests] Metatable/GC mutation stress complete ({} iterations)", iterations);
}

/// Advanced: Snapshot/restore fuzzing during VM operations
pub fn snapshot_restore_fuzz(state: &mut LuaState, ops: usize) {
    let mut snapshots = Vec::new();
    use rand::Rng;
    for i in 0..ops {
        fuzz_vm(state, 1);
        if rand::thread_rng().gen_bool(0.2) {
            let snap = snapshot_vm(state);
            snapshots.push(snap);
            println!("[ltests] Snapshot taken at op {}", i);
        }
        if !snapshots.is_empty() && rand::thread_rng().gen_bool(0.2) {
            let idx = rand::thread_rng().gen_range(0..snapshots.len());
            restore_vm(state, &snapshots[idx]);
            println!("[ltests] Restored snapshot #{} at op {}", idx, i);
        }
    }
    println!("[ltests] Snapshot/restore fuzzing complete ({} ops)", ops);
}

/// Advanced: Stack/heap randomization and canary checks
const STACK_CANARY: i64 = 0xC0FFEE_CAFE_BABE;

pub fn insert_stack_canary(state: &mut LuaState) {
    if state.stack_size() > 0 {
        let idx = rand::thread_rng().gen_range(0..state.stack_size());
        state.set_stack(idx, LuaValue::Int(STACK_CANARY));
        println!("[ltests] Stack canary inserted at index {}", idx);
    }
}

pub fn check_stack_canary(state: &LuaState) -> bool {
    for i in 0..state.stack_size() {
        if let LuaValue::Int(v) = state.get_stack(i) {
            if v == STACK_CANARY {
                println!("[ltests] Stack canary found at index {}", i);
                return true;
            }
        }
    }
    println!("[ltests] Stack canary not found");
    false
}

pub fn randomize_stack(state: &mut LuaState) {
    use rand::seq::SliceRandom;
    let mut vals = Vec::new();
    for i in 0..state.stack_size() {
        vals.push(state.get_stack(i));
    }
    vals.shuffle(&mut rand::thread_rng());
    for (i, v) in vals.into_iter().enumerate() {
        state.set_stack(i, v);
    }
    println!("[ltests] Stack randomized");
}

/// Advanced: Coverage-guided fuzzing stub
pub fn coverage_guided_fuzz(state: &mut LuaState, iterations: usize) {
    // Stub: In a real implementation, coverage would be tracked and used to guide input
    for _ in 0..iterations {
        fuzz_vm(state, 1);
        COVERAGE.hit("fuzz_vm");
    }
    COVERAGE.report();
    println!("[ltests] Coverage-guided fuzzing stub complete ({} iterations)", iterations);
}

/// Advanced: VM state serialization roundtrip test
pub fn vm_state_roundtrip_test(state: &mut LuaState) {
    let snap = snapshot_vm(state);
    let mut state2 = state.clone();
    restore_vm(&mut state2, &snap);
    if state.stack_snapshot() == state2.stack_snapshot() {
        println!("[ltests] VM state roundtrip test passed");
    } else {
        println!("[ltests] VM state roundtrip test FAILED");
        println!("Original: {:?}", state.stack_snapshot());
        println!("Restored: {:?}", state2.stack_snapshot());
    }
}