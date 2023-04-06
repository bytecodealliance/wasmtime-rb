use super::static_id::StaticId;
use magnus::{exception::arg_error, Error, Symbol, Value};
use std::fmt::Display;

/// Represents an enum as a set of Symbols (using `StaticId`).
/// Each symbol maps to a value of the enum's type.
pub struct SymbolEnum<'a, T: Clone> {
    what: &'a str,
    mapping: Mapping<T>,
}

impl<'a, T: Clone> SymbolEnum<'a, T> {
    pub fn new(what: &'a str, mapping: Vec<(StaticId, T)>) -> Self {
        Self {
            what,
            mapping: Mapping(mapping),
        }
    }

    /// Map a Magnus `Value` to the  entry in the enum.
    /// Returns an `ArgumentError` with a message enumerating all valid symbols
    /// when `needle` isn't a valid symbol.
    pub fn get(&self, needle: Value) -> Result<T, magnus::Error> {
        let needle: Symbol = needle.try_convert().map_err(|_| self.error(needle))?;
        let id = magnus::value::Id::from(needle);

        self.mapping
            .0
            .iter()
            .find(|(haystack, _)| *haystack == id)
            .map(|found| found.1.clone())
            .ok_or_else(|| self.error(needle.as_value()))
    }

    pub fn error(&self, value: Value) -> Error {
        Error::new(
            arg_error(),
            format!(
                "invalid {}, expected one of {}, got {:?}",
                self.what, self.mapping, value
            ),
        )
    }
}

struct Mapping<T: Clone>(Vec<(StaticId, T)>);

/// Mimicks `Array#inpsect`'s output with all valid symbols.
/// E.g.: `[:s1, :s2, :s3]`
impl<T: Clone> Display for Mapping<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        if let Some(((last, _), elems)) = self.0.split_last() {
            for (id, _) in elems.iter() {
                write!(f, ":{}, ", Symbol::from(*id).name().unwrap())?;
            }
            write!(f, ":{}", Symbol::from(*last).name().unwrap())?;
        }

        write!(f, "]")?;
        Ok(())
    }
}
