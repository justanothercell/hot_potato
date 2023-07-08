#![feature(unboxed_closures)]
#![feature(tuple_trait)]
#![feature(fn_traits)]
#![feature(exit_status_error)]

use std::{marker::Tuple, process::{Command, ExitStatusError}};
use parking_lot::RwLock;

pub use hot_potato_proc_macro::potato;
pub use inventory::submit;

pub struct PotatoFunc<Args: Tuple, Output>  {
    path: &'static str,
    func: Option<RwLock<Box<dyn Fn<Args, Output = Output>>>>
}

impl<Args: Tuple, Output> PotatoFunc<Args, Output> {
    /// Safety:
    /// DO NOT USE MANUALLY!
    pub const unsafe fn new(path: &'static str) -> Self {
        let potato = Self { 
            path: path, 
            func: None
        };
        potato
    }

    pub const fn handle(&self) -> PotatoHandle{
        PotatoHandle {  }
    }
}

pub struct PotatoHandle {

}

inventory::collect!(PotatoHandle);

#[must_use]
pub fn build_and_reload_potatoes() -> Result<(), ExitStatusError> {
    let mut compile = Command::new("cargo").args(["rustc", "--lib", "--", "--crate-type", "cdylib"]).spawn().expect("Could not launch compilation");
    let status = compile.wait().expect("did nto complete successfully");
    status.exit_ok()?;

    for potato_handle in inventory::iter::<PotatoHandle> {

    };
    Ok(())
}

impl<Args: Tuple, Output> FnOnce<Args> for PotatoFunc<Args, Output> {
    type Output = Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.func.as_ref().map(|f| f.read().call(args)).expect("function was not loaded")
    }
}

impl<Args: Tuple, Output> FnMut<Args> for PotatoFunc<Args, Output> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.func.as_ref().map(|f| f.read().call(args)).expect("function was not loaded")
    }
}

impl<Args: Tuple, Output> Fn<Args> for PotatoFunc<Args, Output> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.func.as_ref().map(|f| f.read().call(args)).expect("function was not loaded")
    }
}


unsafe impl <Args: Tuple, Output> Sync for PotatoFunc<Args, Output> {}