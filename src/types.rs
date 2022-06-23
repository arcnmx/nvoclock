use clap::ArgMatches;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GpuDescriptor {
    pub name: String,
}

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Copy, Clone)]
pub enum ResetSettings {
    VoltageBoost,
    SensorLimits,
    PowerLimits,
    CoolerLevels,
    VfpDeltas,
    VfpLock,
    PStateDeltas,
    Overvolt,
}

pub const POSSIBLE_BOOL_OFF: &'static str = "off";
pub const POSSIBLE_BOOL_ON: &'static str = "on";
pub const POSSIBLE_BOOL: &'static [&'static str] = &[POSSIBLE_BOOL_OFF, POSSIBLE_BOOL_ON];

pub fn parse_bool_match(matches: &ArgMatches, arg: &'static str) -> bool {
    // Weird interaction with defaults and empty values
    let occ = matches.occurrences_of(arg);
    let values = matches.values_of(arg).map(|v| v.count()).unwrap_or(0);
    let value = matches.value_of(arg);

    let v = match (occ, values, value) {
        (1, 1, Some(..)) => POSSIBLE_BOOL_ON,
        (0, 1, Some(v)) => v,
        (1, 2, Some(v)) => v,
        _ => unreachable!(),
    };

    match v {
        POSSIBLE_BOOL_OFF => false,
        POSSIBLE_BOOL_ON => true,
        _ => unreachable!(),
    }
}
