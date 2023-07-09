#![feature(unboxed_closures)]
#![feature(tuple_trait)]
#![feature(fn_traits)]
#![feature(exit_status_error)]

use std::{marker::Tuple, process::{Command, Stdio}, collections::HashMap, any::Any};
use libloading::{Library, Symbol};
use parking_lot::{RwLock, RwLockWriteGuard};

pub use hot_potato_proc_macro::potato;
pub use inventory::submit;

pub struct PotatoFunc<Args: Tuple, MagicArgs: Tuple, Output>  {
    path: &'static str,
    func: RwLock<Option<Box<dyn Fn<MagicArgs, Output = Output>>>>,
    magics: RwLock<Option<HashMap<&'static str, Box<dyn Any>>>>,
    initializer: Option<for<'a> fn(RwLockWriteGuard<'a, Option<HashMap<&'static str, Box<dyn Any>>>>)>,
    mapper: for<'a> fn(Args, &Self) -> MagicArgs
}

impl<Args: Tuple, MagicArgs: Tuple, Output> PotatoFunc<Args, MagicArgs, Output> {
    /// Safety:
    /// DO NOT USE MANUALLY! Only meant for macro use!
    pub const unsafe fn new(path: &'static str, initializer: for<'a> fn(RwLockWriteGuard<'a, Option<HashMap<&'static str, Box<dyn Any>>>>), mapper: for<'a> fn(Args, &Self) -> MagicArgs) -> Self {
        let potato = Self { 
            path: path, 
            func: RwLock::new(None),
            magics: RwLock::new(None),
            initializer: Some(initializer),
            mapper
        };
        potato
    }

    /// Safety:
    /// DO NOT USE MANUALLY! Only meant for macro use!
    pub const unsafe fn handle(&self) -> PotatoHandle{
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
    pub fn get<T: Clone + 'static>(&self, magic: &str) -> T {
        let reader = self.magics.read();
        let map = reader.as_ref().expect("function was not initalized");
        let any = map.get(magic).expect(&format!("invalid magic `{}` does not exist!", magic));
        let v: &T = any.downcast_ref().expect("type mismatch while getting magic!");
        v.clone()
    }

    pub fn set<T: Clone + 'static>(&self, magic: &'static str, v: T){
        // easiest way to get all the checks for free!
        self.get::<T>(magic);

        let mut writer = self.magics.write();
        let map = writer.as_mut().expect("function was not initalized");
        map.insert(magic, Box::new(v));
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
    let status = compile.wait().expect("did not complete successfully");
    status.exit_ok().map_err(|e| e.to_string())?;

    let next = std::env::args().next().unwrap();
    let (base, _) = next.rsplit_once(std::path::MAIN_SEPARATOR).unwrap();

    #[cfg(windows)]
    let lib = format!("potato_dytarget.dll");
    #[cfg(not(windows))]
    let lib = "libpotato_dytarget.so";

    let potato_lib = unsafe { Library::new(format!("{base}/{lib}")).map_err(|e| e.to_string())? };

    for potato_handle in inventory::iter::<PotatoHandle> {
        (potato_handle.loader)(potato_handle.potato, &potato_lib);
        let potato = unsafe { &mut *(potato_handle.potato as *mut PotatoFunc<(), (), ()>) };
        if let Some(initializer) = potato.initializer.take() {
            initializer(potato.magics.write())
        }
    }

    unsafe { LIBHOLDER = Some(potato_lib); }

    Ok(())
}

impl<Args: Tuple, MagicArgs: Tuple, Output> FnOnce<Args> for PotatoFunc<Args, MagicArgs, Output> {
    type Output = Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call((self.mapper)(args, &self))).expect("function was not loaded")
    }
}

impl<Args: Tuple, MagicArgs: Tuple, Output> FnMut<Args> for PotatoFunc<Args, MagicArgs, Output> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call((self.mapper)(args, self))).expect("function was not loaded")
    }
}

impl<Args: Tuple, MagicArgs: Tuple, Output> Fn<Args> for PotatoFunc<Args, MagicArgs, Output> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call((self.mapper)(args, self))).expect("function was not loaded")
    }
}


unsafe impl <Args: Tuple, MagicArgs: Tuple, Output> Sync for PotatoFunc<Args, MagicArgs, Output> {}