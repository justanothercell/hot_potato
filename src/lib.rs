#![feature(unboxed_closures)]
#![feature(tuple_trait)]
#![feature(fn_traits)]

use std::marker::{Tuple, PhantomData};

pub use hot_potato_proc_macro::potato;

pub struct PotatoFunc<Args: Tuple, Output>  {
    pub func: Option<Box<dyn FnMut<Args, Output = Output>>>
}

impl<Args: Tuple, Output> FnOnce<Args> for PotatoFunc<Args, Output> {
    type Output = Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.func.map(|f| f.call_once(args)).expect("function was not loaded")
    }
}

impl<Args: Tuple, Output> FnMut<Args> for PotatoFunc<Args, Output> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.func.as_mut().map(|f| f.call_mut(args)).expect("function was not loaded")
    }
}

impl<Args: Tuple, Output> Fn<Args> for PotatoFunc<Args, Output> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.func.as_mut().map(|f| f.call_mut(args)).expect("function was not loaded")
    }
}


unsafe impl <Args: Tuple, Output> Sync for PotatoFunc<Args, Output> {}