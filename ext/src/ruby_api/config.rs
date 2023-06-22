mod tracked_memory_creator;
pub(crate) use self::tracked_memory_creator::TrackedMemoryCreator;
use crate::{define_rb_intern, helpers::SymbolEnum};
use lazy_static::lazy_static;
use magnus::{
    exception::{arg_error, type_error},
    prelude::*,
    r_hash::ForEach,
    Error, RHash, Symbol, TryConvert, Value,
};
use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};
use wasmtime::{Config, OptLevel, ProfilingStrategy, Strategy, WasmBacktraceDetails};

define_rb_intern!(
    DEBUG_INFO => "debug_info",
    WASM_BACKTRACE_DETAILS => "wasm_backtrace_details",
    NATIVE_UNWIND_INFO => "native_unwind_info",
    CONSUME_FUEL => "consume_fuel",
    EPOCH_INTERRUPTION => "epoch_interruption",
    MAX_WASM_STACK => "max_wasm_stack",
    WASM_THREADS => "wasm_threads",
    WASM_MULTI_MEMORY => "wasm_multi_memory",
    WASM_MEMORY64 => "wasm_memory64",
    PROFILER => "profiler",
    CRANELIFT_OPT_LEVEL => "cranelift_opt_level",
    STRATEGY => "strategy",
    PARALLEL_COMPILATION => "parallel_compilation",
    NONE => "none",
    JITDUMP => "jitdump",
    VTUNE => "vtune",
    PERFMAP => "perfmap",
    SPEED => "speed",
    SPEED_AND_SIZE => "speed_and_size",
    TARGET => "target",
    GENERATE_ADDRESS_MAP => "generate_address_map",
    AUTO => "auto",
    CRANELIFT => "cranelift",
    WINCH => "winch",
);

lazy_static! {
    static ref OPT_LEVEL_MAPPING: SymbolEnum<'static, OptLevel> = {
        let mapping = vec![
            (*NONE, OptLevel::None),
            (*SPEED, OptLevel::Speed),
            (*SPEED_AND_SIZE, OptLevel::SpeedAndSize),
        ];

        SymbolEnum::new(":cranelift_opt_level", mapping)
    };
    static ref PROFILING_STRATEGY_MAPPING: SymbolEnum<'static, ProfilingStrategy> = {
        let mapping = vec![
            (*NONE, ProfilingStrategy::None),
            (*JITDUMP, ProfilingStrategy::JitDump),
            (*VTUNE, ProfilingStrategy::VTune),
            (*PERFMAP, ProfilingStrategy::PerfMap),
        ];

        SymbolEnum::new(":profiler", mapping)
    };
    static ref STRATEGY_MAPPING: SymbolEnum<'static, Strategy> = {
        let mapping = vec![
            (*AUTO, Strategy::Auto),
            (*CRANELIFT, Strategy::Cranelift),
            (*WINCH, Strategy::Winch),
        ];

        SymbolEnum::new(":strategy", mapping)
    };
}

/// Default for [`wasmtime::Config`], which includes a [`TrackedMemoryCreator`]
/// to report memory usage to Ruby.
pub fn default_config() -> Config {
    let mut config = Config::new();
    let host_memory = TrackedMemoryCreator::new();
    config.with_host_memory(Arc::new(host_memory));
    config
}

pub fn hash_to_config(hash: RHash) -> Result<Config, Error> {
    let mut config = default_config();
    hash.foreach(|name: Symbol, value: Value| {
        let id = magnus::value::Id::from(name);
        let entry = ConfigEntry(name, value);

        if *DEBUG_INFO == id {
            config.debug_info(entry.try_into()?);
        } else if *WASM_BACKTRACE_DETAILS == id {
            config.wasm_backtrace_details(entry.try_into()?);
        } else if *NATIVE_UNWIND_INFO == id {
            config.native_unwind_info(entry.try_into()?);
        } else if *CONSUME_FUEL == id {
            config.consume_fuel(entry.try_into()?);
        } else if *EPOCH_INTERRUPTION == id {
            config.epoch_interruption(entry.try_into()?);
        } else if *MAX_WASM_STACK == id {
            config.max_wasm_stack(entry.try_into()?);
        } else if *WASM_THREADS == id {
            config.wasm_threads(entry.try_into()?);
        } else if *WASM_MULTI_MEMORY == id {
            config.wasm_multi_memory(entry.try_into()?);
        } else if *WASM_MEMORY64 == id {
            config.wasm_memory64(entry.try_into()?);
        } else if *PARALLEL_COMPILATION == id {
            config.parallel_compilation(entry.try_into()?);
        } else if *PROFILER == id {
            config.profiler(entry.try_into()?);
        } else if *CRANELIFT_OPT_LEVEL == id {
            config.cranelift_opt_level(entry.try_into()?);
        } else if *STRATEGY == id && cfg!(feature = "winch") {
            config.strategy(entry.try_into()?);
        } else if *TARGET == id {
            let target: Option<String> = entry.try_into()?;

            if let Some(target) = target {
                config.target(&target).map_err(|e| {
                    Error::new(arg_error(), format!("Invalid target: {}: {}", target, e))
                })?;
            }
        } else if *GENERATE_ADDRESS_MAP == id {
            config.generate_address_map(entry.try_into()?);
        } else {
            return Err(Error::new(
                arg_error(),
                format!("Unknown option: {}", name.inspect()),
            ));
        }

        Ok(ForEach::Continue)
    })?;

    Ok(config)
}

struct ConfigEntry(Symbol, Value);

impl ConfigEntry {
    fn invalid_type(&self) -> Error {
        Error::new(
            type_error(),
            format!("Invalid option {}: {}", self.1, self.0),
        )
    }
}

impl TryFrom<ConfigEntry> for bool {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Self::Error> {
        Self::try_convert(value.1).map_err(|_| value.invalid_type())
    }
}

impl TryFrom<ConfigEntry> for usize {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Self::Error> {
        Self::try_convert(value.1).map_err(|_| value.invalid_type())
    }
}

impl TryFrom<ConfigEntry> for String {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Self::Error> {
        Self::try_convert(value.1).map_err(|_| value.invalid_type())
    }
}

impl TryFrom<ConfigEntry> for Option<String> {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Self::Error> {
        let val: Option<String> = value.1.try_convert().map_err(|_| value.invalid_type())?;
        Ok(val)
    }
}

impl TryFrom<ConfigEntry> for WasmBacktraceDetails {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<WasmBacktraceDetails, Error> {
        Ok(match value.1.to_bool() {
            true => WasmBacktraceDetails::Enable,
            false => WasmBacktraceDetails::Disable,
        })
    }
}

impl TryFrom<ConfigEntry> for wasmtime::ProfilingStrategy {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Error> {
        PROFILING_STRATEGY_MAPPING.get(value.1)
    }
}

impl TryFrom<ConfigEntry> for wasmtime::OptLevel {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Error> {
        OPT_LEVEL_MAPPING.get(value.1)
    }
}

impl TryFrom<ConfigEntry> for wasmtime::Strategy {
    type Error = magnus::Error;
    fn try_from(value: ConfigEntry) -> Result<Self, Error> {
        STRATEGY_MAPPING.get(value.1)
    }
}
