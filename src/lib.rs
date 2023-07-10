#![feature(unboxed_closures)]
#![feature(tuple_trait)]
#![feature(fn_traits)]
#![feature(exit_status_error)]

use std::{marker::Tuple, process::{Command, Stdio}, collections::HashMap, any::Any, fmt::Debug};
use libloading::{Library, Symbol};
use parking_lot::{RwLock, RwLockWriteGuard};

pub use hot_potato_proc_macro::potato;
pub use inventory::submit;

type InitFn = for<'a> fn(RwLockWriteGuard<'a, Option<HashMap<&'static str, Box<dyn Any>>>>);

pub struct PotatoFunc<Args: Tuple, MagicArgs: Tuple, Output, F: Fn<MagicArgs, Output = Output> + Copy>  {
    path: &'static str,
    func: RwLock<Option<Box<dyn Fn<MagicArgs, Output = Output>>>>,
    magics: RwLock<Option<HashMap<&'static str, Box<dyn Any>>>>,
    initializer: Option<InitFn>,
    mapper: for<'a> fn(Args, &Self) -> MagicArgs,
    _dummy: Option<F>
}

impl<Args: Tuple, MagicArgs: Tuple, Output, F: Fn<MagicArgs, Output = Output> + Copy> PotatoFunc<Args, MagicArgs, Output, F> {
    /// # Safety
    /// DO NOT USE MANUALLY! Only meant for macro use!
    pub const unsafe fn new(path: &'static str, initializer: InitFn, mapper: for<'a> fn(Args, &Self) -> MagicArgs) -> Self {
        
        Self { 
            path, 
            func: RwLock::new(None),
            magics: RwLock::new(None),
            initializer: Some(initializer),
            mapper,
            _dummy: None
        }
    }

    /// # Safety
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
        let fun: Symbol<F> = unsafe { potato_lib.get(self.path.as_ref()).expect("could not find potatoed function") };
        let boxed: Box<dyn Fn<MagicArgs, Output = Output>> = Box::new(*fun);
        let boxed: Box<dyn Fn<MagicArgs, Output = Output>> = unsafe{ std::mem::transmute(boxed) };

        let mut func = self.func.write();
        *func = Some(boxed);
    }
    pub fn get<T: Clone + 'static>(&self, magic: &str) -> T {
        let reader = self.magics.read();
        let map = reader.as_ref().expect("function was not initalized");
        let any = map.get(magic).unwrap_or_else(|| panic!("invalid magic `{}` does not exist!", magic));
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

pub fn build_and_reload_potatoes() -> Result<(), String> {
    // aquire write locks so that no one tries to run a function while the lib is reloading
    let mut locks = vec![];
    for potato_handle in inventory::iter::<PotatoHandle> {
        let potato = unsafe { &mut *(potato_handle.potato as *mut PotatoFunc<(), (), (), fn()>) };
        locks.push((potato.magics.write(), potato_handle));
    }

    // drop old lib
    unsafe { LIBHOLDER.take(); }

    let mut compile = Command::new("cargo").args(["rustc", "--lib", "--", "--crate-type", "cdylib"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn().map_err(|e| e.to_string())?;
    let status = compile.wait().expect("did not complete successfully");
    status.exit_ok().map_err(|e| e.to_string())?;

    let next = std::env::args().next().unwrap();
    let (base, exe) = next.rsplit_once(std::path::MAIN_SEPARATOR).unwrap();
    let name = exe.rsplit_once('.').unwrap_or((exe, "")).0;

    #[cfg(windows)]
    let lib = format!("{name}.dll");
    #[cfg(not(windows))]
    let lib = format!("lib{name}.so");

    let potato_lib = unsafe { Library::new(format!("{base}/{lib}")).map_err(|e| e.to_string())? };

    for (magics, potato_handle) in locks {
        (potato_handle.loader)(potato_handle.potato, &potato_lib);
        let potato = unsafe { &mut *(potato_handle.potato as *mut PotatoFunc<(), (), (), fn()>) };
        if let Some(initializer) = potato.initializer.take() {
            initializer(magics)
        }
    }

    unsafe { LIBHOLDER = Some(potato_lib); }

    Ok(())
}

impl<Args: Tuple, MagicArgs: Tuple, Output, F: Fn<MagicArgs, Output = Output> + Copy> FnOnce<Args> for PotatoFunc<Args, MagicArgs, Output, F> {
    type Output = Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call((self.mapper)(args, &self))).expect("function was not loaded")
    }
}

impl<Args: Tuple, MagicArgs: Tuple, Output, F: Fn<MagicArgs, Output = Output> + Copy> FnMut<Args> for PotatoFunc<Args, MagicArgs, Output, F> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.func.read().as_ref().map(|f| f.call((self.mapper)(args, self))).expect("function was not loaded")
    }
}

impl<Args: Tuple + Debug, MagicArgs: Tuple + Debug, Output, F: Fn<MagicArgs, Output = Output> + Copy> Fn<Args> for PotatoFunc<Args, MagicArgs, Output, F> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        let margs = (self.mapper)(args, self);
        self.func.read().as_ref().map(|f| f.call(margs)).expect("function was not loaded")
    }
}


unsafe impl <Args: Tuple, MagicArgs: Tuple, Output, F: Fn<MagicArgs, Output = Output> + Copy> Sync for PotatoFunc<Args, MagicArgs, Output, F> {}