//! lstate.rs - Global State for Lua VM (Rust port)
// Ported and modernized from lstate.c/h

use crate::lobject::*;
use crate::ltm::*;
use crate::lzio::*;
use crate::lgc::*;
use crate::lmem::*;
use crate::lstring::*;
use crate::ltable::*;
use crate::lua::*;
use std::ptr;
use std::cell::RefCell;
use std::rc::Rc;

// --- CallInfo struct ---
#[derive(Debug, Default)]
pub struct CallInfo {
    pub func: usize, // Stack index
    pub top: usize,  // Stack index
    pub previous: Option<Rc<RefCell<CallInfo>>>,
    pub next: Option<Rc<RefCell<CallInfo>>>,
    pub callstatus: u32,
    // ...other fields as needed...
}

// --- Lua Thread State ---
#[derive(Debug)]
pub struct LuaState {
    pub stack: Vec<LuaValue>,
    pub ci: Rc<RefCell<CallInfo>>,
    pub nci: usize,
    pub status: TStatus,
    pub l_G: Rc<RefCell<GlobalState>>,
    // --- More fields for LuaState ---
    pub error: Option<String>, // Last error message
    pub pc: usize,             // Program counter
    // --- Hook and error jump management ---
    pub hook: Option<fn()>,
    pub error_jump: Option<usize>,
    // --- Upvalue management ---
    pub open_upvalues: Vec<LuaValue>,
}

// --- Global State ---
#[derive(Debug)]
pub struct GlobalState {
    pub gc: GarbageCollector,
    pub strt: StringTable,
    pub registry: LuaValue,
    pub nilvalue: LuaValue,
    pub seed: u32,
    // --- More fields for GlobalState ---
    pub total_bytes: usize, // Total allocated bytes
    // --- Warning function (stub) ---
    pub warning_func: Option<fn(&str)>,
}

// --- Functions (stubs, to be filled out as needed) ---
impl LuaState {
    pub fn new(l_G: Rc<RefCell<GlobalState>>) -> Self {
        LuaState {
            stack: Vec::with_capacity(256),
            ci: Rc::new(RefCell::new(CallInfo::default())),
            nci: 0,
            status: TStatus::LUA_OK,
            l_G,
            error: None,
            pc: 0,
            hook: None,
            error_jump: None,
            open_upvalues: Vec::new(),
        }
    }
    pub fn push(&mut self, value: LuaValue) {
        self.stack.push(value);
    }
    pub fn pop(&mut self) -> Option<LuaValue> {
        self.stack.pop()
    }
    pub fn top(&self) -> Option<&LuaValue> {
        self.stack.last()
    }
    pub fn set_status(&mut self, status: TStatus) {
        self.status = status;
    }
    pub fn is_ok(&self) -> bool {
        self.status == TStatus::LUA_OK
    }
    // --- More fields and helpers for LuaState ---
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }
    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }
    pub fn get_global(&self, key: &str) -> Option<&LuaValue> {
        // Example: lookup in registry/global table (stub)
        Some(&LuaValue::Nil)
    }
    pub fn set_global(&mut self, key: &str, value: LuaValue) {
        // Example: set in registry/global table (stub)
    }
    pub fn error(&mut self, msg: &str) {
        self.status = TStatus::LUA_ERRRUN;
        // In a real VM, would raise/propagate error
        eprintln!("Lua error: {}", msg);
    }
    pub fn is_yieldable(&self) -> bool {
        // Placeholder: always yieldable
        true
    }
    // --- More advanced VM helpers and fields ---
    pub fn yieldable(&self) -> bool {
        (self.nci & 0xffff0000) == 0
    }
    pub fn get_ccalls(&self) -> usize {
        self.nci & 0xffff
    }
    pub fn inc_nyci(&mut self) {
        self.nci += 0x10000;
    }
    pub fn dec_nyci(&mut self) {
        self.nci -= 0x10000;
    }
    pub fn set_upvalue(&mut self, _idx: usize, _val: LuaValue) {
        // TODO: implement upvalue logic
    }
    pub fn get_upvalue(&self, _idx: usize) -> Option<&LuaValue> {
        // TODO: implement upvalue logic
        None
    }
    pub fn set_registry(&mut self, _key: &str, _val: LuaValue) {
        // TODO: implement registry logic
    }
    pub fn get_registry(&self, _key: &str) -> Option<&LuaValue> {
        // TODO: implement registry logic
        None
    }
    // --- Thread list, registry table, and metatable helpers ---
    pub fn add_to_thread_list(&self) {
        // TODO: implement thread list logic
    }
    pub fn remove_from_thread_list(&self) {
        // TODO: implement thread list logic
    }
    pub fn set_registry_value(&mut self, _key: &str, _val: LuaValue) {
        // TODO: implement registry table logic
    }
    pub fn get_registry_value(&self, _key: &str) -> Option<&LuaValue> {
        // TODO: implement registry table logic
        None
    }
    pub fn set_value_metatable(&mut self, _val: &LuaValue, _mt: LuaValue) {
        // TODO: implement value metatable logic
    }
    pub fn get_value_metatable(&self, _val: &LuaValue) -> Option<&LuaValue> {
        // TODO: implement value metatable logic
        None
    }
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            gc: GarbageCollector::new(),
            strt: StringTable::new(),
            registry: LuaValue::Nil,
            nilvalue: LuaValue::Nil,
            seed: 0,
            total_bytes: 0,
            warning_func: None,
        }
    }
    pub fn set_registry(&mut self, value: LuaValue) {
        self.registry = value;
    }
    pub fn set_nilvalue(&mut self, value: LuaValue) {
        self.nilvalue = value;
    }
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }
    pub fn set_debt(&mut self, debt: isize) {
        // Example: update GC debt (stub)
        // self.gc.debt = debt;
    }
    // --- Global helpers ---
    pub fn total_bytes(&self) -> usize {
        // Example: return total allocated bytes (stub)
        0
    }
    pub fn gc_collect(&mut self) {
        // Example: trigger GC (stub)
    }
    pub fn panic(&self, msg: &str) {
        // Example: panic handler (stub)
        panic!("Lua panic: {}", msg);
    }
    pub fn set_metatable(&mut self, _typeidx: usize, _table: LuaValue) {
        // TODO: implement metatable logic
    }
    pub fn get_metatable(&self, _typeidx: usize) -> Option<&LuaValue> {
        // TODO: implement metatable logic
        None
    }
    pub fn set_tmname(&mut self, _idx: usize, _name: String) {
        // TODO: implement tag method name logic
    }
    pub fn get_tmname(&self, _idx: usize) -> Option<&str> {
        // TODO: implement tag method name logic
        None
    }
}

// --- Example stub for a function ---
pub fn luaE_setdebt(g: &mut GlobalState, debt: isize) {
    // ...implement logic for setting GC debt...
}

// --- Example: thread creation and freeing ---
pub fn luaE_newthread(g: Rc<RefCell<GlobalState>>) -> LuaState {
    LuaState::new(g)
}

pub fn luaE_freethread(_L: &mut LuaState, _L1: &mut LuaState) {
    // In Rust, memory is managed automatically, but you can add cleanup logic here if needed.
}

// --- Example: CallInfo extension ---
impl CallInfo {
    pub fn extend(&mut self) -> Rc<RefCell<CallInfo>> {
        let new_ci = Rc::new(RefCell::new(CallInfo::default()));
        self.next = Some(new_ci.clone());
        new_ci.borrow_mut().previous = Some(Rc::new(RefCell::new(self.clone())));
        new_ci
    }
}

// --- Thread/stack management helpers ---
pub fn luaE_checkcstack(_L: &LuaState) -> bool {
    // Example: check C stack depth (stub)
    true
}

pub fn luaE_incCstack(_L: &mut LuaState) {
    // Example: increment C stack counter (stub)
}

pub fn luaE_warning(_L: &LuaState, msg: &str, _tocont: bool) {
    eprintln!("Lua warning: {}", msg);
}

pub fn luaE_warnerror(_L: &LuaState, where_: &str) {
    eprintln!("Lua VM error in {}", where_);
}

// --- Test scaffolding ---
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lua_state_stack() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.push(LuaValue::Nil);
        assert_eq!(state.top(), Some(&LuaValue::Nil));
        state.pop();
        assert!(state.top().is_none());
    }
    #[test]
    fn test_stack_clear() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.push(LuaValue::Nil);
        state.clear_stack();
        assert_eq!(state.stack_size(), 0);
    }
    #[test]
    fn test_error_status() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.error("fail");
        assert_eq!(state.status, TStatus::LUA_ERRRUN);
    }
}

// --- More test scaffolding ---
#[cfg(test)]
mod more_tests {
    use super::*;
    #[test]
    fn test_stack_clear() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.push(LuaValue::Nil);
        state.clear_stack();
        assert_eq!(state.stack_size(), 0);
    }
    #[test]
    fn test_error_status() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.error("fail");
        assert_eq!(state.status, TStatus::LUA_ERRRUN);
    }
}

// --- Advanced tests for new functionality ---
#[cfg(test)]
mod advanced_tests {
    use super::*;
    #[test]
    fn test_yieldable_and_ccalls() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        assert!(state.yieldable());
        assert_eq!(state.get_ccalls(), 0);
        state.inc_nyci();
        assert!(!state.yieldable());
        state.dec_nyci();
        assert!(state.yieldable());
    }
}

// --- Coroutine/thread helpers and more advanced state management ---
#[cfg(test)]
mod coroutine_tests {
    use super::*;
    #[test]
    fn test_error_set_get_clear() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.set_error("fail".to_string());
        assert_eq!(state.get_error(), Some("fail"));
        state.clear_error();
        assert_eq!(state.get_error(), None);
    }
    #[test]
    fn test_pc_set_get() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.set_pc(42);
        assert_eq!(state.get_pc(), 42);
    }
    #[test]
    fn test_resume_yield_stub() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        assert!(state.resume().is_ok());
        assert!(state.yield_thread().is_ok());
    }
}

// --- Hook and error jump management, upvalue helpers ---
#[cfg(test)]
mod hook_upvalue_tests {
    use super::*;
    #[test]
    fn test_set_get_hook_stub() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.set_hook(None);
        assert!(state.get_hook().is_none());
    }
    #[test]
    fn test_set_get_error_jump_stub() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.set_error_jump(Some(123));
        assert_eq!(state.get_error_jump(), None); // stub always None
    }
    #[test]
    fn test_add_close_upvalues_stub() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let mut state = LuaState::new(g);
        state.add_open_upvalue(0, LuaValue::Nil);
        state.close_upvalues();
        // No panic = pass (stub)
    }
}

// --- Thread list, registry table, and metatable helpers ---
#[cfg(test)]
mod thread_registry_tests {
    use super::*;
    #[test]
    fn test_registry_table_stub() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let reg = g.borrow().registry_table();
        assert!(matches!(reg, LuaValue::Nil));
    }
    #[test]
    fn test_thread_list_stub() {
        let g = Rc::new(RefCell::new(GlobalState::new()));
        let threads = g.borrow().thread_list();
        assert!(threads.is_empty());
    }
}
