use nvapi::{PState, CoolerPolicy, ClockDomain};
use crate::types::{ResetSettings, OutputFormat};
use crate::Error;

pub trait ConvertEnum: Sized {
    fn from_str(s: &str) -> Result<Self, Error>;
    fn to_str(&self) -> &'static str;
    fn possible_values() -> &'static [&'static str];
    fn possible_values_typed() -> &'static [Self];
}

macro_rules! enum_from_str {
    (
        $conv:ident => {
        $(
            $item:ident = $str:expr,
        )*
            _ => $err:expr,
        }
    ) => {
        impl ConvertEnum for $conv {
            fn from_str(s: &str) -> Result<Self, Error> {
                match s {
                $(
                    $str => Ok($conv::$item),
                )*
                    _ => Err(($err).into()),
                }
            }

            #[allow(unreachable_patterns)]
            fn to_str(&self) -> &'static str {
                match *self {
                $(
                    $conv::$item => $str,
                )*
                    _ => "unknown",
                }
            }

            fn possible_values() -> &'static [&'static str] {
                &[$(
                    $str,
                )*]
            }

            fn possible_values_typed() -> &'static [Self] {
                &[$(
                    $conv::$item,
                )*]
            }
        }
    };
}

enum_from_str! {
    OutputFormat => {
        Human = "human",
        Json = "json",
        _ => "unknown output format",
    }
}

enum_from_str! {
    ResetSettings => {
        VoltageBoost = "voltage-boost",
        SensorLimits = "thermal",
        PowerLimits = "power",
        CoolerLevels = "cooler",
        VfpDeltas = "vfp",
        VfpLock = "lock",
        PStateDeltas = "pstate",
        Overvolt = "overvolt",
        _ => "unknown setting",
    }
}

enum_from_str! {
    PState => {
        P0 = "P0",
        P1 = "P1",
        P2 = "P2",
        P3 = "P3",
        P4 = "P4",
        P5 = "P5",
        P6 = "P6",
        P7 = "P7",
        P8 = "P8",
        P9 = "P9",
        P10 = "P10",
        P11 = "P11",
        P12 = "P12",
        P13 = "P13",
        P14 = "P14",
        P15 = "P15",
        _ => "unknown pstate",
    }
}

enum_from_str! {
    ClockDomain => {
        Graphics = "graphics",
        Memory = "memory",
        Processor = "processor",
        Video = "video",
        _ => "unknown clock type",
    }
}

enum_from_str! {
    CoolerPolicy => {
        None = "default",
        Manual = "manual",
        Performance = "perf",
        TemperatureDiscrete = "discrete",
        TemperatureContinuous = "continuous",
        Hybrid = "hybrid",
        TemperatureContinuousSoftware = "software",
        Default = "default32",
        _ => "unknown cooler policy",
    }
}
