use super::{
    convert::{ToSym, ToValType},
    root,
};
use magnus::{function, method, Error, Module as _, Object, RArray, Symbol};
use wasmtime::{FuncType as FuncTypeImpl, ValType};

/// @yard
/// Represents a Func's signature.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.FuncType.html Wasmtime's Rust doc
#[derive(Clone, Debug)]
#[magnus::wrap(class = "Wasmtime::FuncType")]
pub struct FuncType {
    inner: FuncTypeImpl,
}

impl FuncType {
    pub fn get(&self) -> &FuncTypeImpl {
        &self.inner
    }
}

impl FuncType {
    /// @yard
    /// A descriptor for a function in a WebAssembly module.
    /// WebAssembly functions can have 0 or more parameters and results. Each param
    /// must be a valid WebAssembly type represented as a symbol. The valid symbols are:
    /// +:i32+, +:i64+, +:f32+, +:f64+, +:v128+, +:funcref+, +:externref+.
    ///
    /// @def new(params, results)
    /// @param params [Array<Symbol>] The function's parameter types.
    /// @param results [Array<Symbol>] The function's result types.
    /// @return [FuncType]
    ///
    /// @example +FuncType+ that takes 2 +i32+s and returns 1 +i32+:
    ///   FuncType.new([:i32, :i32], [:i32])
    pub fn new(params: RArray, results: RArray) -> Result<Self, Error> {
        let inner = FuncTypeImpl::new(params.to_val_type_vec()?, results.to_val_type_vec()?);
        Ok(Self { inner })
    }

    /// @yard
    /// @return [Array<Symbol>] The function's parameter types.
    pub fn params(&self) -> Vec<Symbol> {
        self.get().params().map(ToSym::to_sym).collect()
    }

    /// @yard
    /// @return [Array<Symbol>] The function's result types.
    pub fn results(&self) -> Vec<Symbol> {
        self.get().results().map(ToSym::to_sym).collect()
    }
}

trait ToValTypeVec {
    fn to_val_type_vec(&self) -> Result<Vec<ValType>, Error>;
}

impl ToValTypeVec for RArray {
    fn to_val_type_vec(&self) -> Result<Vec<ValType>, Error> {
        unsafe { self.as_slice() }
            .iter()
            .map(ToValType::to_val_type)
            .collect::<Result<Vec<ValType>, Error>>()
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("FuncType", Default::default())?;

    class.define_singleton_method("new", function!(FuncType::new, 2))?;
    class.define_method("params", method!(FuncType::params, 0))?;
    class.define_method("results", method!(FuncType::results, 0))?;

    Ok(())
}
