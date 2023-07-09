#![feature(unboxed_closures)]
#![feature(tuple_trait)]
#![feature(fn_traits)]
#![feature(exit_status_error)]

use std::{marker::Tuple, process::{Command, Stdio}};
use libloading::{Library, Symbol};
use parking_lot::RwLock;

pub use hot_potato_proc_macro::potato;
pub use inventory::submit;

pub struct PotatoFunc<Args: Tuple, Output>  {
    path: &'static str,
    func: RwLock<Option<Box<dyn Fn<Args, Output = Output>>>>
}

impl<Args: Tuple, Output> PotatoFunc<Args, Output> {
    /// Safety:
    /// DO NOT USE MANUALLY!
    pub const unsafe fn new(path: &'static str) -> Self {
        let potato = Self { 
            path: path, 
            func: RwLock::new(None)
        };
        potato
    }

    pub const fn handle(&self) -> PotatoHandle{
        PotatoHandle { 
            potato: self as *const _ as *const u8,
            loader: |potato, potato_lib| {
                let potato = unsafe { &mut *(potato as *mut Self) };
                potato.load_from_lib(potato_lib)
            }
        }
    }

    fn load_from_lib(&self, potato_lib: &Library) {
        let fun: Symbol<fn()> = unsafe { potato_lib.get(self.path.as_ref()).expect("could not find potatoed function") };
        let fun: Symbol<fn(Args) -> Output> = unsafe{ std::mem::transmute(fun) };
        let boxed: Box<dyn Fn<(Args,), Output = Output>> = Box::new(*fun);
        let boxed: Box<dyn Fn<Args, Output = Output>> = unsafe{ std::mem::transmute(boxed) };

        let mut func = self.func.write();
        *func = Some(unsafe{ std::mem::transmute(boxed) });
    }
}

pub struct PotatoHandle {
    potato: *const u8,
    loader: fn(*const u8, &Library)
}

unsafe impl Sync for PotatoHandle {}

inventory::collect!(PotatoHandle);

static mut LIBHOLDER: Option<Library> = None;

#[must_use]
pub fn build_and_reload_potatoes() -> Result<(), String> {
    // drop old lib
    unsafe { LIBHOLDER.take(); }

    let mut compile = Command::new("cargo").args(["rustc", "--lib", "--", "--crate-type", "cdylib"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn().map_err(|e| e.to_string())?;
    let status = compile.wait().expect("did nto complete successfully");
    status.exit_ok().map_err(|e| e.to_string())?;

    let next = std::env::args().next().unwrap();
    let (base, exe) = next.rsplit_once(std::path::MAIN_SEPARATOR).unwrap();
    let name = exe.rsplit_once(".").unwrap_or((exe, "")).0;

    #[cfg(windows)]
    let lib = format!("{name}.dll");
    #[cfg(not(windows))]
    let lib = format!("lib{name}.so");

    let potato_lib = unsafe { Library::new(format!("{base}/{lib}")).map_err(|e| e.to_string())? };

    for potato_handle in inventory::iter::<PotatoHandle> {
        (potato_handle.loader)(potato_handle.potato, &potato_lib)
    }

    unsafe { LIBHOLDER = Some(potato_lib); }

    Ok(())
}

impl<Args: Tuple, Output> FnOnce<Args> for PotatoFunc<Args, Output> {
    type Output = Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call(args)).expect("function was not loaded")
    }
}

impl<Args: Tuple, Output> FnMut<Args> for PotatoFunc<Args, Output> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call(args)).expect("function was not loaded")
    }
}

impl<Args: Tuple, Output> Fn<Args> for PotatoFunc<Args, Output> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call(args)).expect("function was not loaded")
    }
}


unsafe impl <Args: Tuple, Output> Sync for PotatoFunc<Args, Output> {}