use std::{boxed, cell::Cell, error::Error, fmt::Display, rc::Rc};

use crate::{Luau, _LuaState};

pub struct LuauThread {
    root_check: Rc<Cell<bool>>,
    thread: *mut Luau,
}

#[derive(Debug)]
pub struct MainStateDeadError;

impl Error for MainStateDeadError {}

impl Display for MainStateDeadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Attempted to get LuauThread inner state when the main state has died"
        )
    }
}

impl LuauThread {
    pub unsafe fn from_ptr(state: *mut _LuaState, root_check: Rc<Cell<bool>>) -> Self {
        let boxed_luau = Box::new(Luau::from_ptr(state));
        
        Self {
            root_check,
            thread: Box::into_raw(boxed_luau)
        }
    }

    pub fn try_get_state(&self) -> Result<&Luau, MainStateDeadError> {
        if self.root_check.get() == false {
            Err(MainStateDeadError {})
        } else {
            Ok(unsafe { &*self.thread })
        }
    }

    pub fn get_state(&self) -> &Luau {
        self.try_get_state().unwrap()
    }
}
