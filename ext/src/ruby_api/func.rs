use super::{root, store::Store};
use magnus::{
    block::Proc, function, gc, DataTypeFunctions, Error, Module as _, Object, RArray, TypedData,
    Value,
};
use wasmtime::{AsContextMut, Extern, Func as FuncImpl, FuncType};

#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Func", mark, size)]
pub struct Func {
    store: Value,
    proc: Value,
    inner: FuncImpl,
}

impl DataTypeFunctions for Func {
    fn mark(&self) {
        gc::mark(&self.store);
        gc::mark(&self.proc);
    }
}

// Wraps a Proc to satisfy wasmtime::Func's Send+Sync requirements. This is safe
// to do as long as (1) we hold the GVL when whe execute the proc and (2) we do
// not have multiple threads running at once (e.g. with Wasm thread proposal).
struct ShareableProc(magnus::block::Proc);
unsafe impl Send for ShareableProc {}
unsafe impl Sync for ShareableProc {}

unsafe impl Send for Func {}

impl Func {
    pub fn new(s: Value, _ty: Value, _caller: bool, proc: Proc) -> Result<Self, Error> {
        // TODOs:
        // - Deal with functype (params and args)
        // - Handle exceptions. Idea: return a wasmtime::TrapReason::Error that
        //   wraps the Ruby exception?  Should we raise that error directly to the
        //   consumer, or should it be a Trap exception with a trap `cause?
        // - Inject the caller (always? or depending on _caller? Would work nicely as a kwarg).

        let store: &Store = s.try_convert()?;
        let mut store = store.borrow_mut();
        let context = store.as_context_mut();
        let ty = FuncType::new(vec![], vec![]); // TODO
        let shareable_proc = ShareableProc(proc);

        let inner = wasmtime::Func::new(context, ty, move |_caller, _params, _results| {
            let arr = RArray::new();
            shareable_proc.0.call::<RArray, Value>(arr);
            Ok(())
        });

        Ok(Self {
            store: s,
            proc: proc.into(),
            inner,
        })
    }

    pub fn get(&self) -> FuncImpl {
        // hmm? Loses ownership?
        self.inner
    }
}

impl From<&Func> for Extern {
    fn from(func: &Func) -> Self {
        Self::Func(func.get())
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Func", Default::default())?;
    class.define_singleton_method("new", function!(Func::new, 4))?;

    Ok(())
}
