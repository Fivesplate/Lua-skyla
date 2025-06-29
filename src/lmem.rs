//! lmem.rs - Memory manager for Lua VM (Rust translation of lmem.c)

use crate::lstate::{lua_State, global_State};
use crate::llimits::*;
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;
use std::alloc::{System, GlobalAlloc};
use crate::lgc::{luaC_fullgc, luaC_step};

/// Minimum size for arrays during parsing
pub const MINSIZEARRAY: usize = 4;

/// Threshold for triggering incremental GC step (tune as needed)
pub const GCDEBT_THRESHOLD: l_mem = 1024 * 1024; // 1MB for example

/// Memory allocation error
pub fn luaM_toobig(L: &mut lua_State) -> ! {
    panic!("memory allocation error: block too big");
}

/// Free memory
pub unsafe fn luaM_free(L: &mut lua_State, block: *mut u8, osize: usize) {
    let g = L.global();
    debug_assert!((osize == 0) == (block.is_null()));
    if !block.is_null() {
        let layout = Layout::from_size_align_unchecked(osize, LUAI_MAXALIGN);
        dealloc(block, layout);
        g.GCdebt += osize as l_mem;
    }
}

/// Allocate memory
pub unsafe fn luaM_malloc(L: &mut lua_State, size: usize) -> *mut u8 {
    if size == 0 {
        ptr::null_mut()
    } else {
        let g = L.global();
        let layout = Layout::from_size_align_unchecked(size, LUAI_MAXALIGN);
        let mut newblock = alloc(layout);
        if newblock.is_null() {
            // Try full GC and retry allocation once
            luaC_fullgc(L, false);
            newblock = alloc(layout);
            if newblock.is_null() {
                luaM_toobig(L);
            }
        }
        g.GCdebt -= size as l_mem;
        // Trigger incremental GC step if debt is high
        if g.GCdebt < -GCDEBT_THRESHOLD {
            luaC_step(L);
        }
        newblock
    }
}

/// Reallocate memory (generic allocation routine)
pub unsafe fn luaM_realloc(L: &mut lua_State, block: *mut u8, osize: usize, nsize: usize) -> *mut u8 {
    let g = L.global();
    debug_assert!((osize == 0) == (block.is_null()));
    let mut newblock = if block.is_null() {
        if nsize == 0 {
            ptr::null_mut()
        } else {
            let layout = Layout::from_size_align_unchecked(nsize, LUAI_MAXALIGN);
            alloc(layout)
        }
    } else if nsize == 0 {
        let layout = Layout::from_size_align_unchecked(osize, LUAI_MAXALIGN);
        dealloc(block, layout);
        ptr::null_mut()
    } else {
        let old_layout = Layout::from_size_align_unchecked(osize, LUAI_MAXALIGN);
        realloc(block, old_layout, nsize)
    };
    if newblock.is_null() && nsize > 0 {
        // Try full GC and retry allocation once
        luaC_fullgc(L, false);
        let layout = Layout::from_size_align_unchecked(nsize, LUAI_MAXALIGN);
        newblock = alloc(layout);
        if newblock.is_null() {
            luaM_toobig(L);
        }
    }
    if !newblock.is_null() {
        g.GCdebt -= nsize as l_mem - osize as l_mem;
        if g.GCdebt < -GCDEBT_THRESHOLD {
            luaC_step(L);
        }
    }
    newblock
}

/// Safe realloc (panics on allocation failure)
pub unsafe fn luaM_saferealloc(L: &mut lua_State, block: *mut u8, osize: usize, nsize: usize) -> *mut u8 {
    let newblock = luaM_realloc(L, block, osize, nsize);
    if newblock.is_null() && nsize > 0 {
        luaM_toobig(L);
    }
    newblock
}

/// Grow an array for the parser
pub unsafe fn luaM_growaux<T>(L: &mut lua_State, block: *mut T, nelems: usize, psize: &mut usize, limit: usize, what: &str) -> *mut T {
    let size = *psize;
    if nelems + 1 <= size {
        return block;
    }
    let newsize = if size >= limit / 2 {
        if size >= limit {
            panic!("too many {} (limit is {})", what, limit);
        }
        limit
    } else {
        let mut s = size * 2;
        if s < MINSIZEARRAY { s = MINSIZEARRAY; }
        s
    };
    debug_assert!(nelems + 1 <= newsize && newsize <= limit);
    let newblock = luaM_saferealloc(L, block as *mut u8, size * std::mem::size_of::<T>(), newsize * std::mem::size_of::<T>()) as *mut T;
    *psize = newsize;
    newblock
}

/// Shrink a vector to its final size
pub unsafe fn luaM_shrinkvector<T>(L: &mut lua_State, block: *mut T, size: &mut usize, final_n: usize) -> *mut T {
    let oldsize = *size * std::mem::size_of::<T>();
    let newsize = final_n * std::mem::size_of::<T>();
    debug_assert!(newsize <= oldsize);
    let newblock = luaM_saferealloc(L, block as *mut u8, oldsize, newsize) as *mut T;
    *size = final_n;
    newblock
}

/// Reallocate memory with alignment
pub unsafe fn luaM_realloc_aligned(L: &mut lua_State, block: *mut u8, osize: usize, nsize: usize, align: usize) -> *mut u8 {
    let g = L.global();
    let old_layout = Layout::from_size_align_unchecked(osize, align);
    let mut ptr = realloc(block, old_layout, nsize);
    if ptr.is_null() && nsize > 0 {
        luaC_fullgc(L, false);
        let layout = Layout::from_size_align_unchecked(nsize, align);
        ptr = alloc(layout);
        if ptr.is_null() {
            luaM_toobig(L);
        }
    }
    g.GCdebt -= nsize as l_mem - osize as l_mem;
    if g.GCdebt < -GCDEBT_THRESHOLD {
        luaC_step(L);
    }
    ptr
}

/// Allocate zero-initialized memory (like calloc)
pub unsafe fn luaM_calloc(L: &mut lua_State, count: usize, size: usize) -> *mut u8 {
    let total = count.checked_mul(size).expect("overflow in calloc");
    let g = L.global();
    let layout = Layout::from_size_align_unchecked(total, LUAI_MAXALIGN);
    let mut ptr = alloc(layout);
    if ptr.is_null() {
        luaC_fullgc(L, false);
        ptr = alloc(layout);
        if ptr.is_null() {
            luaM_toobig(L);
        }
    }
    if !ptr.is_null() {
        std::ptr::write_bytes(ptr, 0, total);
        g.GCdebt -= total as l_mem;
        if g.GCdebt < -GCDEBT_THRESHOLD {
            luaC_step(L);
        }
    }
    ptr
}

/// Macro-like helper for freeing memory (for API compatibility)
#[inline(always)]
pub unsafe fn luaM_free_vec<T>(L: &mut lua_State, block: *mut T, n: usize) {
    luaM_free(L, block as *mut u8, n * std::mem::size_of::<T>());
}

/// Macro-like helper for allocating a vector
#[inline(always)]
pub unsafe fn luaM_new_vec<T>(L: &mut lua_State, n: usize) -> *mut T {
    luaM_malloc(L, n * std::mem::size_of::<T>()) as *mut T
}

/// Macro-like helper for reallocating a vector
#[inline(always)]
pub unsafe fn luaM_realloc_vec<T>(L: &mut lua_State, block: *mut T, on: usize, n: usize) -> *mut T {
    luaM_realloc(L, block as *mut u8, on * std::mem::size_of::<T>(), n * std::mem::size_of::<T>()) as *mut T
}

/// Macro-like helper for safe reallocating a vector
#[inline(always)]
pub unsafe fn luaM_saferealloc_vec<T>(L: &mut lua_State, block: *mut T, on: usize, n: usize) -> *mut T {
    luaM_saferealloc(L, block as *mut u8, on * std::mem::size_of::<T>(), n * std::mem::size_of::<T>()) as *mut T
}

/// Macro-like helper for allocating a single object
#[inline(always)]
pub unsafe fn luaM_new<T>(L: &mut lua_State) -> *mut T {
    luaM_malloc(L, std::mem::size_of::<T>()) as *mut T
}

/// Macro-like helper for freeing a single object
#[inline(always)]
pub unsafe fn luaM_free_obj<T>(L: &mut lua_State, obj: *mut T) {
    luaM_free(L, obj as *mut u8, std::mem::size_of::<T>());
}

/// Use a more permissive global allocator (System allocator)
#[global_allocator]
static GLOBAL: System = System;
