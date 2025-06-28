//! Rust translation of lfunc.c and lfunc.h
//! Auxiliary functions to manipulate prototypes and closures

// --- lfunc.h translation ---

// Constants and type aliases
pub const MAXUPVAL: u8 = 255;
pub const MAXMISS: usize = 10;
pub const CLOSEKTOP: i32 = lua_sys::LUA_ERRERR + 1; // Assuming lua_sys::LUA_ERRERR is defined

// Helper macros as functions
#[inline]
pub fn size_cclosure(n: usize) -> usize {
    std::mem::size_of::<CClosure>() + std::mem::size_of::<TValue>() * n
}
#[inline]
pub fn size_lclosure(n: usize) -> usize {
    std::mem::size_of::<LClosure>() + std::mem::size_of::<*mut UpVal>() * n
}
#[inline]
pub fn isintwups(L: &lua_State) -> bool {
    !std::ptr::eq(L.twups, L)
}
#[inline]
pub fn upisopen(up: &UpVal) -> bool {
    !std::ptr::eq(up.v.p, &up.u.value as *const _ as *mut _)
}
#[inline]
pub fn uplevel(up: &UpVal) -> StkId {
    debug_assert!(upisopen(up));
    up.v.p as StkId
}

// --- End lfunc.h translation ---

// --- lfunc.c translation ---

impl lua_State {
    pub fn new_cclosure(&mut self, nupvals: usize) -> Box<CClosure> {
        // Allocation logic, replace with actual GC logic as needed
        let c = Box::new(CClosure::new(nupvals));
        c
    }

    pub fn new_lclosure(&mut self, nupvals: usize) -> Box<LClosure> {
        let mut c = Box::new(LClosure::new(nupvals));
        c.p = std::ptr::null_mut();
        c.nupvalues = nupvals as u8;
        for upval in c.upvals.iter_mut() {
            *upval = std::ptr::null_mut();
        }
        c
    }

    pub fn init_upvals(&mut self, cl: &mut LClosure) {
        for i in 0..cl.nupvalues as usize {
            let uv = Box::into_raw(Box::new(UpVal::closed_nil()));
            cl.upvals[i] = uv;
            // luaC_objbarrier(self, cl, uv); // GC barrier, implement as needed
        }
    }

    pub fn find_upval(&mut self, level: StkId) -> *mut UpVal {
        // ...implement logic similar to C code...
        std::ptr::null_mut() // placeholder
    }

    pub fn new_tbcupval(&mut self, level: StkId) {
        // ...implement logic...
    }

    pub fn close_upval(&mut self, level: StkId) {
        // ...implement logic...
    }

    pub fn close(&mut self, level: StkId, status: TStatus, yy: i32) -> StkId {
        // ...implement logic...
        level // placeholder
    }

    pub fn unlink_upval(uv: &mut UpVal) {
        // ...implement logic...
    }
}

impl Proto {
    pub fn new_proto(L: &mut lua_State) -> Box<Proto> {
        Box::new(Proto::default())
    }

    pub fn proto_size(&self) -> usize {
        std::mem::size_of::<Proto>()
            + self.sizep * std::mem::size_of::<*mut Proto>()
            + self.sizek * std::mem::size_of::<TValue>()
            + self.sizelocvars * std::mem::size_of::<LocVar>()
            + self.sizeupvalues * std::mem::size_of::<Upvaldesc>()
            // Add code/lineinfo/abslineinfo if not PF_FIXED
    }

    pub fn free_proto(L: &mut lua_State, f: Box<Proto>) {
        // ...free logic...
    }

    pub fn get_local_name(&self, local_number: i32, pc: i32) -> Option<&str> {
        let mut count = local_number;
        for lv in &self.locvars {
            if lv.startpc <= pc && pc < lv.endpc {
                count -= 1;
                if count == 0 {
                    return Some(lv.varname.as_str());
                }
            }
        }
        None
    }
}

// ...existing code...
    luaD_callnoyield(L, func, 0);
}


/*
** Check whether object at given level has a close metamethod and raise
** an error if not.
*/
static void checkclosemth (lua_State *L, StkId level) {
  const TValue *tm = luaT_gettmbyobj(L, s2v(level), TM_CLOSE);
  if (ttisnil(tm)) {  /* no metamethod? */
    int idx = cast_int(level - L->ci->func.p);  /* variable index */
    const char *vname = luaG_findlocal(L, L->ci, idx, NULL);
    if (vname == NULL) vname = "?";
    luaG_runerror(L, "variable '%s' got a non-closable value", vname);
  }
}


/*
** Prepare and call a closing method.
** If status is CLOSEKTOP, the call to the closing method will be pushed
** at the top of the stack. Otherwise, values can be pushed right after
** the 'level' of the upvalue being closed, as everything after that
** won't be used again.
*/
static void prepcallclosemth (lua_State *L, StkId level, TStatus status,
                                            int yy) {
  TValue *uv = s2v(level);  /* value being closed */
  TValue *errobj;
  switch (status) {
    case LUA_OK:
      L->top.p = level + 1;  /* call will be at this level */
      /* FALLTHROUGH */
    case CLOSEKTOP:  /* don't need to change top */
      errobj = NULL;  /* no error object */
      break;
    default:  /* 'luaD_seterrorobj' will set top to level + 2 */
      errobj = s2v(level + 1);  /* error object goes after 'uv' */
      luaD_seterrorobj(L, status, level + 1);  /* set error object */
      break;
  }
  callclosemethod(L, uv, errobj, yy);
}


/* Maximum value for deltas in 'tbclist' */
#define MAXDELTA       USHRT_MAX


/*
** Insert a variable in the list of to-be-closed variables.
*/
void luaF_newtbcupval (lua_State *L, StkId level) {
  lua_assert(level > L->tbclist.p);
  if (l_isfalse(s2v(level)))
    return;  /* false doesn't need to be closed */
  checkclosemth(L, level);  /* value must have a close method */
  while (cast_uint(level - L->tbclist.p) > MAXDELTA) {
    L->tbclist.p += MAXDELTA;  /* create a dummy node at maximum delta */
    L->tbclist.p->tbclist.delta = 0;
  }
  level->tbclist.delta = cast(unsigned short, level - L->tbclist.p);
  L->tbclist.p = level;
}


void luaF_unlinkupval (UpVal *uv) {
  lua_assert(upisopen(uv));
  *uv->u.open.previous = uv->u.open.next;
  if (uv->u.open.next)
    uv->u.open.next->u.open.previous = uv->u.open.previous;
}


/*
** Close all upvalues up to the given stack level.
*/
void luaF_closeupval (lua_State *L, StkId level) {
  UpVal *uv;
  StkId upl;  /* stack index pointed by 'uv' */
  while ((uv = L->openupval) != NULL && (upl = uplevel(uv)) >= level) {
    TValue *slot = &uv->u.value;  /* new position for value */
    lua_assert(uplevel(uv) < L->top.p);
    luaF_unlinkupval(uv);  /* remove upvalue from 'openupval' list */
    setobj(L, slot, uv->v.p);  /* move value to upvalue slot */
    uv->v.p = slot;  /* now current value lives here */
    if (!iswhite(uv)) {  /* neither white nor dead? */
      nw2black(uv);  /* closed upvalues cannot be gray */
      luaC_barrier(L, uv, slot);
    }
  }
}


/*
** Remove first element from the tbclist plus its dummy nodes.
*/
static void poptbclist (lua_State *L) {
  StkId tbc = L->tbclist.p;
  lua_assert(tbc->tbclist.delta > 0);  /* first element cannot be dummy */
  tbc -= tbc->tbclist.delta;
  while (tbc > L->stack.p && tbc->tbclist.delta == 0)
    tbc -= MAXDELTA;  /* remove dummy nodes */
  L->tbclist.p = tbc;
}


/*
** Close all upvalues and to-be-closed variables up to the given stack
** level. Return restored 'level'.
*/
StkId luaF_close (lua_State *L, StkId level, TStatus status, int yy) {
  ptrdiff_t levelrel = savestack(L, level);
  luaF_closeupval(L, level);  /* first, close the upvalues */
  while (L->tbclist.p >= level) {  /* traverse tbc's down to that level */
    StkId tbc = L->tbclist.p;  /* get variable index */
    poptbclist(L);  /* remove it from list */
    prepcallclosemth(L, tbc, status, yy);  /* close variable */
    level = restorestack(L, levelrel);
  }
  return level;
}


Proto *luaF_newproto (lua_State *L) {
  GCObject *o = luaC_newobj(L, LUA_VPROTO, sizeof(Proto));
  Proto *f = gco2p(o);
  f->k = NULL;
  f->sizek = 0;
  f->p = NULL;
  f->sizep = 0;
  f->code = NULL;
  f->sizecode = 0;
  f->lineinfo = NULL;
  f->sizelineinfo = 0;
  f->abslineinfo = NULL;
  f->sizeabslineinfo = 0;
  f->upvalues = NULL;
  f->sizeupvalues = 0;
  f->numparams = 0;
  f->flag = 0;
  f->maxstacksize = 0;
  f->locvars = NULL;
  f->sizelocvars = 0;
  f->linedefined = 0;
  f->lastlinedefined = 0;
  f->source = NULL;
  return f;
}


lu_mem luaF_protosize (Proto *p) {
  lu_mem sz = cast(lu_mem, sizeof(Proto))
            + cast_uint(p->sizep) * sizeof(Proto*)
            + cast_uint(p->sizek) * sizeof(TValue)
            + cast_uint(p->sizelocvars) * sizeof(LocVar)
            + cast_uint(p->sizeupvalues) * sizeof(Upvaldesc);
  if (!(p->flag & PF_FIXED)) {
    sz += cast_uint(p->sizecode) * sizeof(Instruction);
    sz += cast_uint(p->sizelineinfo) * sizeof(lu_byte);
    sz += cast_uint(p->sizeabslineinfo) * sizeof(AbsLineInfo);
  }
  return sz;
}


void luaF_freeproto (lua_State *L, Proto *f) {
  if (!(f->flag & PF_FIXED)) {
    luaM_freearray(L, f->code, cast_sizet(f->sizecode));
    luaM_freearray(L, f->lineinfo, cast_sizet(f->sizelineinfo));
    luaM_freearray(L, f->abslineinfo, cast_sizet(f->sizeabslineinfo));
  }
  luaM_freearray(L, f->p, cast_sizet(f->sizep));
  luaM_freearray(L, f->k, cast_sizet(f->sizek));
  luaM_freearray(L, f->locvars, cast_sizet(f->sizelocvars));
  luaM_freearray(L, f->upvalues, cast_sizet(f->sizeupvalues));
  luaM_free(L, f);
}


/*
** Look for n-th local variable at line 'line' in function 'func'.
** Returns NULL if not found.
*/
const char *luaF_getlocalname (const Proto *f, int local_number, int pc) {
  int i;
  for (i = 0; i<f->sizelocvars && f->locvars[i].startpc <= pc; i++) {
    if (pc < f->locvars[i].endpc) {  /* is variable active? */
      local_number--;
      if (local_number == 0)
        return getstr(f->locvars[i].varname);
    }
  }
  return NULL;  /* not found */
}

