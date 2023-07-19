use std::iter;
use nvapi::{
    Gpu, GpuInfo, GpuStatus, GpuSettings,
    Celsius, KilohertzDelta, VfPoint,
    ClockDomain, ClockFrequencies, VoltageDomain, Microvolts, PState,
    FanCoolerId, CoolerInfo, CoolerStatus, CoolerSettings, CoolerControl,
    SensorDesc, SensorLimit, SensorThrottle, PStateLimit,
    Utilizations, UtilizationDomain,
    nvapi::{GSyncDevice, GSyncCapabilities, GSyncStatus},
};
use prettytable::{format, row, cell, Table};

const HEADER_LEN: usize = 20;

macro_rules! pline {
    ($header:expr, $($tt:tt)*) => {
        {
            let mut header = $header.to_string();
            while header.len() < HEADER_LEN {
                header.push('.');
            }
            print!("{}: ", header);
            println!($($tt)*);
        }
    };
}

fn table_format() -> format::TableFormat {
    let mut format = format::TableFormat::new();
    format.padding(1, 1);
    format.borders('║');
    format.column_separator('│');
    format.separator(format::LinePosition::Top, format::LineSeparator::new('═', '╤', '╔', '╗'));
    format.separator(format::LinePosition::Title, format::LineSeparator::new('═', '╪', '╠', '╣'));
    format.separator(format::LinePosition::Intern, format::LineSeparator::new('─', '┼', '╟', '╢'));
    format.separator(format::LinePosition::Bottom, format::LineSeparator::new('═', '╧', '╚', '╝'));
    format
}

fn n_a() -> String {
    "N/A".into()
}

pub fn print_settings(set: &GpuSettings) {
    if let Some(ref boost) = set.voltage_boost {
        pline!("Voltage Boost", "{}", boost);
    }
    for limit in &set.sensor_limits {
        pline!("Thermal Limit", "{}{}{}",
            limit.value,
            match &limit.curve {
                Some(pff) => format!(": {}", pff),
                None => n_a(),
            },
            if limit.remove_tdp_limit {
                " (TDP Limit Removed)"
            } else {
                ""
            });
    }
    for limit in &set.power_limits {
        pline!("Power Limit", "{}", limit);
    }
    for (id, cooler) in &set.coolers {
        pline!(format!("Cooler {}", id), "{}", match cooler.level {
            Some(level) => level.to_string(),
            None => cooler.policy.to_string(),
        });
    }
    for (pstate, clock, delta) in set.pstate_deltas.iter().flat_map(|(ps, d)| d.iter().map(move |(clock, d)| (ps, clock, d))) {
        pline!(format!("{} @ {} Offset", clock, pstate), "{}", delta);
    }
    for ov in &set.overvolt {
        pline!("Overvolt", "{}", ov);
    }
    for (id, lock) in &set.vfp_locks {
        if let Some(value) = lock.lock_value {
            pline!(format!("VFP Lock {}", id), "{}", value);
        }
    }
}

    /*let format = table_format();

    if show_vfp {
    }*/

pub fn print_status(status: &GpuStatus) {
    pline!("Power State", "{}", status.pstate);
    pline!("Power Usage", "{}", 
        status.power.iter().fold(None, |state, (ch, power)| if let Some(state) = state {
            Some(format!("{}, {} ({})", state, power, ch))
        } else {
            Some(format!("{} ({})", power, ch))
        }).unwrap_or_else(n_a)
    );
    if let Some(memory) = &status.memory {
        pline!("Memory Usage", "{:.2} / {:.2} ({} evictions totalling {:.2})",
            memory.dedicated_available - memory.dedicated_available_current,
            memory.dedicated_available,
            memory.dedicated_evictions, memory.dedicated_evictions_size,
        );
    }
    if status.ecc.enabled {
        pline!("ECC Errors", "{} 1-bit, {} 2-bit",
            status.ecc.errors.current.single_bit_errors,
            status.ecc.errors.current.double_bit_errors);
        if status.ecc.errors.current != status.ecc.errors.aggregate {
            pline!("ECC Errors", "{} 1-bit, {} 2-bit (Aggregate)",
                status.ecc.errors.aggregate.single_bit_errors,
                status.ecc.errors.aggregate.double_bit_errors);
        }
    }
    if let Some(lanes) = status.pcie_lanes {
        pline!("PCIe Bus Width", "x{}", lanes);
    }
    pline!("Core Voltage", "{}", status.voltage.map(|v| v.to_string()).unwrap_or_else(n_a));
    pline!("Limits", "{}",
        status.perf.limits.fold(None, |state, v| if let Some(state) = state {
            Some(format!("{}, {}", state, v))
        } else {
            Some(v.to_string())
        }).unwrap_or_else(n_a)
    );
    pline!("VFP Lock", "{}",
        if status.vfp_locks.is_empty() {
            "None".into()
        } else {
            status.vfp_locks.iter().map(|(limit, lock)| format!("{}:{}", limit, lock))
                .collect::<Vec<_>>().join(", ")
        },
    );

    for (clock, freq) in &status.clocks {
        pline!(format!("{} Clock", clock), "{}", freq);
    }

    for (res, util) in &status.utilization {
        pline!(format!("{} Load", res), "{}", util);
    }

    for &(ref sensor, ref temp) in &status.sensors {
        pline!("Sensor", "{} ({} / {})", temp, sensor.controller, sensor.target);
    }

    for (i, cooler) in &status.coolers {
        let variable_control = true; // TODO!!
        let level = match cooler.active {
            true if variable_control => cooler.current_level.to_string(),
            true => "On".into(),
            false => "Off".into(),
        };
        let tach = match cooler.current_tach {
            Some(tach) => format!(" ({})", tach),
            None => String::new(),
        };
        pline!(format!("Cooler {}", i), "{}{}", level, tach);
    }
}

/*
    let format = table_format();

    if show_clocks {
    }

    if show_utilizations {
        let mut utils = Table::new();
        utils.set_format(format.clone());
        utils.set_titles(row!["Resource", "Utilization"]);
        for (res, util) in &status.utilization {
            utils.add_row(row![res, util]);
        }
        utils.print_tty(false);
    }

    if show_sensors {
        let mut sensors = Table::new();
        sensors.set_format(format.clone());
        sensors.set_titles(row!["Sensor", "Target", "Temperature"]);
        for &(ref sensor, ref temp) in &status.sensors {
            sensors.add_row(row![sensor.controller, sensor.target, temp]);
        }
        sensors.print_tty(false);
    }

    if show_coolers {
        let mut coolers = Table::new();
        coolers.set_format(format.clone());
        coolers.set_titles(row!["Cooler", "Target", "Level", "Tachometer", "Mode"]);
        for (i, &(ref cooler, ref entry)) in status.coolers.iter().enumerate() {
            let level = match cooler.control {
                CoolerControl::None => n_a(),
                CoolerControl::Toggle => if entry.active {
                    "On".into()
                } else {
                    "Off".into()
                },
                CoolerControl::Variable => entry.level.to_string(),
            };
            let tach = status.tachometer.as_ref().ok()
                .and_then(|&t| if i == 0 { Some(format!("{} RPM", t)) } else { None })
                .unwrap_or_else(n_a);
            coolers.add_row(row![cooler.kind, cooler.target, level, tach, entry.policy]);
        }
        coolers.print_tty(false);
    }

    if show_vfp {
        if let Ok(ref vfp) = status.vfp {
            let mut vfps = Table::new();
            vfps.set_format(format.clone());
            vfps.set_titles(row!["GPU VFP", "Voltage", "Frequency"]);

            for (i, entry) in &vfp.graphics {
                vfps.add_row(row![i, entry.voltage, entry.frequency]);
            }
            vfps.print_tty(false);

            vfps = Table::new();
            vfps.set_format(format.clone());
            vfps.set_titles(row!["Memory VFP", "Voltage", "Frequency"]);

            for (i, entry) in &vfp.memory {
                vfps.add_row(row![i, entry.voltage, entry.frequency]);
            }
            vfps.print_tty(false);
        }
    }
*/

pub fn print_info(info: &GpuInfo) {
    pline!(format!("GPU {}", info.id), "{} ({})", info.name, info.codename);
    pline!("Architecture", "{} ({})", info.arch, info.gpu_type);
    pline!("Vendor", "{}", info.vendor().unwrap_or_default());
    pline!("GPU Shaders", "{} ({}:{} pipes)",
        info.core_count, info.shader_pipe_count, info.shader_sub_pipe_count);
    if let Some(memory) = &info.memory {
        pline!("Video Memory", "{:.2} {}-bit",
            memory.dedicated, info.ram_bus_width);
    } else {
        pline!("Video Memory", "{} {}-bit",
            n_a(), info.ram_bus_width);
    }
    pline!("Memory Type", "{} ({})",
        info.ram_type, info.ram_maker);
    pline!("Memory Banks", "{} ({} partitions)",
        info.ram_bank_count, info.ram_partition_count);
    if let Some(memory) = &info.memory {
        pline!("Memory Avail", "{:.2}", memory.dedicated_available);
        pline!("Shared Memory", "{:.2} ({:.2} system)",
            memory.shared, memory.system);
    }
    pline!("ECC", "{} ({})",
        if info.ecc.info.enabled { "Yes" } else if info.ecc.info.supported { "Disabled" } else { "N/A" },
        info.ecc.info.configuration);
    pline!("Foundry", "{}", info.foundry);
    pline!("Bus", "{}", info.bus);
    if let Some(ids) = info.bus.bus.pci_ids() {
        pline!("PCI IDs", "{}", ids);
    }
    pline!("BIOS Version", "{}", info.bios_version);
    if let Some(driver_model) = &info.driver_model {
        pline!("Driver Model", "{}", driver_model);
    }
    pline!("Limit Support", "{}",
        info.perf.limits.fold(None, |state, v| if let Some(state) = state {
            Some(format!("{}, {}", state, v))
        } else {
            Some(v.to_string())
        }).unwrap_or_else(|| "None".into())
    );
    if info.vfp_limits.is_empty() {
        pline!("VFP", "No");
    } else {
        for (clock, limit) in &info.vfp_limits {
            pline!(format!("VFP ({})", clock), "{}", limit.range);
        }
    }


    for (_, limit) in info.power_limits.iter().enumerate() {
        pline!("Power Limit", "{} ({} default)", limit.range, limit.default);
    }

    for clock in ClockDomain::values() {
        if let (Some(base), boost) = (info.base_clocks.get(&clock), info.boost_clocks.get(&clock)) {
            pline!(format!("{} Clock", clock), "{} ({} boost)",
                base, boost.map(ToString::to_string).unwrap_or_else(n_a)
            );
        }
    }

    for (_, (sensor, limit)) in info.sensors.iter()
        .zip(info.sensor_limits.iter().map(Some).chain(iter::repeat(None)))
        .enumerate()
    {
        pline!("Thermal Sensor", "{} / {} ({} range)",
            sensor.controller, sensor.target, sensor.range);
        if let Some(limit) = limit {
            pline!("Thermal Limit", "{} ({} default)", limit.range, limit.default);
            if let Some(pff) = &limit.throttle_curve {
                pline!("Thermal Throttle", "{}", pff);
            }
        }
    }

    for (id, cooler) in info.coolers.iter() {
        let range =  match (cooler.default_level_range, cooler.tach_range) {
            (Some(level), Some(tach)) => Some(format!("{} / {}", level, tach)),
            (None, Some(tach)) => Some(tach.to_string()),
            (Some(level), None) => Some(level.to_string()),
            (None, None) => None,
        };
        pline!(format!("Cooler {}", id), "{} / {} / {}{}",
            cooler.kind, cooler.controller, cooler.target,
            match range {
                Some(range) => format!(" ({} range)", range),
                None => match cooler.control {
                    CoolerControl::Variable => "",
                    CoolerControl::Toggle => "(On/Off control)",
                    CoolerControl::None => " (Read-only)",
                    _ => "",
                }.into(),
            },
        );
        if cooler.default_policy != nvapi::CoolerPolicy::None {
            pline!(format!("Cooler {} Default", id), "{} Mode", cooler.default_policy);
        }
    }
}

    /*let format = table_format();

    if show_clocks {
        let mut clocks = Table::new();
        clocks.set_format(format.clone());
        clocks.set_titles(row!["Clock", "Base", "Boost"]);
        for clock in ClockDomain::values() {
            if let (Some(base), boost) = (info.base_clocks.get(&clock), info.boost_clocks.get(&clock)) {
                clocks.add_row(row![clock, base, boost.map(ToString::to_string).unwrap_or_else(n_a)]);
            }
        }
        clocks.print_tty(false);
    }

    if show_coolers {
        let mut coolers = Table::new();
        coolers.set_format(format.clone());
        coolers.set_titles(row!["Cooler", "Controller", "Target", "Range", "Control", "Default Mode"]);
        for (_, cooler) in info.coolers.iter().enumerate() {
            coolers.add_row(row![
                cooler.kind, cooler.controller, cooler.target, cooler.range,
                cooler.control, cooler.default_policy
            ]);
        }
        coolers.print_tty(false);
    }

    if show_sensors {
        let mut sensors = Table::new();
        sensors.set_format(format.clone());
        sensors.set_titles(row!["Thermal Sensor", "Target", "Range", "Limit Range", "Default"]);
        for (_, (sensor, limit)) in info.sensors.iter()
            .zip(info.sensor_limits.iter().map(Some).chain(iter::repeat(None)))
            .enumerate()
        {
            sensors.add_row(row![
                sensor.controller, sensor.target, sensor.range,
                limit.map(|l| l.range.to_string()).unwrap_or_else(n_a),
                limit.map(|l| l.default.to_string()).unwrap_or_else(n_a)
            ]);
        }
        sensors.print_tty(false);
    }

    if show_plimits {
        let mut plimit = Table::new();
        plimit.set_format(format.clone());
        plimit.set_titles(row!["Power Limit", "Default"]);
        for (_, limit) in info.power_limits.iter().enumerate() {
            plimit.add_row(row![limit.range, limit.default]);
        }
        plimit.print_tty(false);
    }

    if show_pstates {
        let mut pstates = Table::new();
        pstates.set_format(format.clone());
        pstates.set_titles(row!["PState", "Clock", "Frequency Range", "Overclock Limits", "Voltage"]);
        for (pstate, clock, limit) in info.pstate_limits.iter().flat_map(|(p, e)| e.iter().map(move |(c, e)| (p, c, e))) {
            pstates.add_row(row![
                pstate, clock, limit.frequency,
                limit.frequency_delta.map(|d| d.to_string()).unwrap_or_else(n_a),
                if limit.voltage_domain == VoltageDomain::Undefined { n_a() } else { limit.voltage_domain.to_string() }
            ]);
        }
        pstates.print_tty(false);
    }*/

pub fn print_clocks(base: &ClockFrequencies, boost: &ClockFrequencies, current: &ClockFrequencies, util: &Utilizations) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["Clock", "Usage", "Current", "Base", "Boost"]);
    for clock in ClockDomain::values() {
        match (
            base.get(&clock), boost.get(&clock), current.get(&clock),
            UtilizationDomain::from_clock(clock).and_then(|u| util.get(&u))
        ) {
            (None, _, None, _) => (),
            (base, boost, current, usage) => {
                table.add_row(row![
                    clock,
                    usage.map(|v| v.to_string()).unwrap_or_else(n_a),
                    current.map(|v| v.to_string()).unwrap_or_else(n_a),
                    base.map(|v| v.to_string()).unwrap_or_else(n_a),
                    boost.map(|v| v.to_string()).unwrap_or_else(n_a)
                ]);
            },
        }
    }
    table.print_tty(false);
}

pub fn print_coolers<'a, I: Iterator<Item=(FanCoolerId, &'a CoolerInfo, &'a CoolerStatus, &'a CoolerSettings)>>(coolers: I) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["Cooler", "Type", "Controller", "Target", "Level", "Speed", "Range", "Setting", "Mode", "Default"]);
    for (id, cooler, status, control) in coolers {
        let (level, range) = match cooler.control {
            CoolerControl::Toggle => (if status.active {
                "On".into()
            } else {
                "Off".into()
            }, "On / Off".into()),
            CoolerControl::Variable => (status.current_level.to_string(), status.current_level_range.to_string()),
            _ => (n_a(), n_a()),
        };
        let tach = match status.current_tach {
            Some(tach) => tach.to_string(),
            None => n_a(),
        };
        let level = match control.level {
            Some(level) => level.to_string(),
            None => n_a(),
        };
        table.add_row(row![id, cooler.kind, cooler.controller, cooler.target, level, tach, range, level, control.policy, cooler.default_policy]);
    }
    table.print_tty(false);
}

pub fn print_sensors<'a, I: Iterator<Item=(&'a SensorDesc, Option<(&'a SensorLimit, &'a SensorThrottle)>, Celsius)>>(sensors: I) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["Sensor", "Target", "Temperature", "Range", "Limit Range", "Default", "Limit"]);
    for (sensor, limit, temp) in sensors {
        let (limit_range, limit_default, limit) = if let Some((desc, limit)) = limit {
            (desc.range.to_string(), desc.default.to_string(), limit.value.to_string())
        } else {
            (n_a(), n_a(), n_a())
        };
        table.add_row(row![
            sensor.controller, sensor.target, temp, sensor.range,
            limit_range, limit_default, limit
        ]);
    }
    table.print_tty(false);
}

pub fn print_vfp<I: Iterator<Item=(usize, VfPoint)>>(clock: ClockDomain, vfp: I, lock: Option<Microvolts>, core: Option<Microvolts>) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row![format!("{}", clock), "Voltage", "Frequency", "Offset", "Default"]);

    for (i, point) in vfp {
        let mut flags = String::new();
        if Some(point.voltage) == core {
            flags.push('*');
        }
        if Some(point.voltage) == lock {
            flags.push('^');
        }

        table.add_row(row![format!("{}{}", i, flags), point.voltage, point.frequency, point.delta, point.default_frequency]);
    }
    table.print_tty(false);
}

pub fn print_pstates<'a, I: Iterator<Item=(PState, ClockDomain, &'a PStateLimit, Option<KilohertzDelta>)>>(pstates: I, current: Option<PState>) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["PState", "Clock", "Frequency Range", "Offset", "Offset Limits", "Voltage"]);
    for (pstate, clock, limit, delta) in pstates {
        let mut flags = String::new();
        if Some(pstate) == current {
            flags.push('*');
        }
        table.add_row(row![
            format!("{}{}", pstate, flags), clock, limit.frequency,
            delta.map(|d| d.to_string()).unwrap_or_else(n_a),
            limit.frequency_delta.map(|d| d.to_string()).unwrap_or_else(n_a),
            if limit.voltage_domain == VoltageDomain::Undefined { n_a() } else { limit.voltage_domain.to_string() }
        ]);
    }
    table.print_tty(false);
}

pub fn print_gsync_status(device: &GSyncDevice, gpu: &Gpu, status: &GSyncStatus) {
    pline!("G-SYNC", "{}", device.handle().as_ptr() as usize);
    pline!("GPU", "{}", gpu.id());
    pline!("Sync", "{:?}", status.synced);
    pline!("Stereo Sync", "{:?}", status.stereo_synced);
    pline!("Signal", "{:?}", status.sync_signal_available);
}

pub fn print_gsync_info(device: &GSyncDevice, capabilities: &GSyncCapabilities) {
    pline!("G-SYNC", "{}", device.handle().as_ptr() as usize);
    let board_id = match capabilities.board_id {
        nvapi::sys::gsync::NVAPI_GSYNC_BOARD_ID_P358 => "P358".into(),
        nvapi::sys::gsync::NVAPI_GSYNC_BOARD_ID_P2060 => "P2060".into(),
        nvapi::sys::gsync::NVAPI_GSYNC_BOARD_ID_P2061 => "P2061".into(),
        id => id.to_string(),
    };
    pline!("Board ID", "{}", capabilities.board_id);
    pline!("Revision", "{}.{}", capabilities.revision, capabilities.extended_revision);
    if let Some(max_mul_div) = capabilities.max_mul_div {
        pline!("Maximum Mul/Div", "{}", max_mul_div);
    }
}
