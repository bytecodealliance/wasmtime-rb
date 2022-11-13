use magnus::{error::Error, exception, gc, value::Value, RTypedData, TryConvert, TypedData};
use std::{marker::PhantomData, ops::Deref};

/// A small wrapper for `RTypedData` that keeps track of the concrete struct
/// type, and the underlying [`Value`] for GC purposes.
#[derive(Debug)]
#[repr(transparent)]
pub struct WrappedStruct<T: TypedData + 'static> {
    inner: Value,
    phantom: PhantomData<&'static T>,
}

impl<T: TypedData> Clone for WrappedStruct<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T: TypedData> WrappedStruct<T> {
    /// Gets the underlying struct.
    pub fn get(&self) -> Result<&T, Error> {
        self.inner.try_convert::<&T>()
    }

    /// Get the Ruby [`Value`] for this struct.
    pub fn to_value(&self) -> Value {
        self.inner
    }

    /// Marks the Ruby [`Value`] for GC.
    pub fn mark(&self) {
        gc::mark(&self.inner);
    }
}

impl<'t, T: TypedData> From<WrappedStruct<T>> for Value {
    fn from(wrapped_struct: WrappedStruct<T>) -> Self {
        wrapped_struct.inner
    }
}

impl<T: TypedData> Deref for WrappedStruct<T> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: TypedData> From<T> for WrappedStruct<T> {
    fn from(t: T) -> Self {
        Self {
            inner: RTypedData::wrap(t).into(),
            phantom: PhantomData,
        }
    }
}

impl<'t, T> TryConvert for WrappedStruct<T>
where
    T: TypedData,
{
    fn try_convert(val: Value) -> Result<Self, Error> {
        let inner = RTypedData::from_value(val).ok_or_else(|| {
            Error::new(
                exception::type_error(),
                format!(
                    "no implicit conversion of {} into {}",
                    unsafe { val.classname() },
                    T::class()
                ),
            )
        })?;

        Ok(Self {
            inner: inner.into(),
            phantom: PhantomData,
        })
    }
}
