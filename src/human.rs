use std::iter;
use nvapi::{
    GpuInfo, GpuStatus, GpuSettings,
    Celsius, KilohertzDelta, VfPoint,
    ClockDomain, ClockFrequencies, VoltageDomain, Microvolts, PState,
    CoolerDesc, CoolerStatus, CoolerControl, ClockLockMode,
    SensorDesc, SensorLimit, PStateLimit,
    Utilizations, UtilizationDomain,
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
        pline!("Thermal Limit", "{}", limit);
    }
    for limit in &set.power_limits {
        pline!("Power Limit", "{}", limit);
    }
    for &(ref desc, ref cooler) in &set.coolers {
        pline!(format!("Cooler {}", desc.kind), "{}", cooler.level);
    }
    for (pstate, clock, delta) in set.pstate_deltas.iter().flat_map(|(ps, d)| d.iter().map(move |(clock, d)| (ps, clock, d))) {
        pline!(format!("{} @ {} Offset", clock, pstate), "{}", delta);
    }
    for ov in &set.overvolt {
        pline!("Overvolt", "{}", ov);
    }
    for (_, lock) in &set.vfp_locks {
        if lock.mode == ClockLockMode::Manual {
            pline!("VFP Lock", "{}", lock.voltage);
        }
    }
}

    /*let format = table_format();

    if show_vfp {
    }*/

pub fn print_status(status: &GpuStatus) {
    pline!("Power State", "{}", status.pstate);
    pline!("Power Usage", "{}", 
        status.power.iter().fold(None, |state, v| if let Some(state) = state {
            Some(format!("{}, {}", state, v))
        } else {
            Some(v.to_string())
        }).unwrap_or_else(n_a)
    );
    pline!("Memory Usage", "{:.2} / {:.2} ({} evictions totalling {:.2})",
        status.memory.dedicated_available - status.memory.dedicated_available_current,
        status.memory.dedicated_available,
        status.memory.dedicated_evictions, status.memory.dedicated_evictions_size,
    );
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
        status.vfp_locks.iter().map(|(_, v)| v).max_by_key(|v| v.0)
            .map(|v| v.to_string()).unwrap_or_else(|| "None".into())
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
        let tach = status.tachometer.as_ref()
            .and_then(|&t| if i == 0 { Some(format!(" ({} RPM)", t)) } else { None })
            .unwrap_or_else(|| String::new());
        pline!(format!("Cooler {}", cooler.kind), "{}{}", level, tach);
        pline!("Cooler Mode", "{}", entry.policy);
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
    pline!("GPU", "{} ({})", info.name, info.codename);
    pline!("Architecture", "{} ({})", info.arch, info.gpu_type);
    pline!("Vendor", "{}", info.vendor().unwrap_or_default());
    pline!("GPU Shaders", "{} ({}:{} pipes)",
        info.core_count, info.shader_pipe_count, info.shader_sub_pipe_count);
    pline!("Video Memory", "{:.2} {}-bit",
        info.memory.dedicated, info.ram_bus_width);
    pline!("Memory Type", "{} ({})",
        info.ram_type, info.ram_maker);
    pline!("Memory Banks", "{} ({} partitions)",
        info.ram_bank_count, info.ram_partition_count);
    pline!("Memory Avail", "{:.2}", info.memory.dedicated_available);
    pline!("Shared Memory", "{:.2} ({:.2} system)",
        info.memory.shared, info.memory.system);
    pline!("ECC", "{} ({})",
        if info.ecc.info.enabled { "Yes" } else if info.ecc.info.supported { "Disabled" } else { "Unupported" },
        info.ecc.info.configuration);
    pline!("Foundry", "{}", info.foundry);
    pline!("Bus", "{}", info.bus);
    if let Some(ids) = info.bus.bus.pci_ids() {
        pline!("PCI IDs", "{}", ids);
    }
    pline!("BIOS Version", "{}", info.bios_version);
    pline!("Driver Model", "{}", info.driver_model);
    pline!("Limit Support", "{}",
        info.perf.limits.fold(None, |state, v| if let Some(state) = state {
            Some(format!("{}, {}", state, v))
        } else {
            Some(v.to_string())
        }).unwrap_or_else(|| "None".into())
    );
    pline!("VFP Support", "{}",
        if info.vfp_limits.is_empty() { "No" } else { "Yes" });

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
        pline!("Thermal Limit", "{} ({} default)",
            limit.map(|l| l.range.to_string()).unwrap_or_else(n_a),
            limit.map(|l| l.default.to_string()).unwrap_or_else(n_a),
        );
    }

    for (_, cooler) in info.coolers.iter().enumerate() {
        pline!(format!("Cooler {}", cooler.kind), "{} / {} ({} range)",
            cooler.controller, cooler.target,
            match cooler.control {
                CoolerControl::Variable => cooler.range.to_string(),
                CoolerControl::Toggle => "On/Off".into(),
                CoolerControl::None => n_a(),
            },
        );
        pline!("Cooler Default", "{} Mode", cooler.default_policy);
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

pub fn print_coolers<'a, I: Iterator<Item=(&'a CoolerDesc, &'a CoolerStatus)>>(coolers: I, tach: Option<u32>) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["Cooler", "Controller", "Target", "Level", "RPM", "Range", "Mode", "Default"]);
    for (i, (cooler, status)) in coolers.enumerate() {
        let (level, range) = match cooler.control {
            CoolerControl::None => (n_a(), n_a()),
            CoolerControl::Toggle => (if status.active {
                "On".into()
            } else {
                "Off".into()
            }, "On / Off".into()),
            CoolerControl::Variable => (status.level.to_string(), cooler.range.to_string()),
        };
        let tach = tach.and_then(|t| if i == 0 { Some(t.to_string()) } else { None }).unwrap_or_else(n_a);
        table.add_row(row![cooler.kind, cooler.controller, cooler.target, level, tach, range, status.policy, cooler.default_policy]);
    }
    table.print_tty(false);
}

pub fn print_sensors<'a, I: Iterator<Item=(&'a SensorDesc, Option<(&'a SensorLimit, Celsius)>, Celsius)>>(sensors: I) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["Sensor", "Target", "Temperature", "Range", "Limit Range", "Default", "Limit"]);
    for (sensor, limit, temp) in sensors {
        let (limit_range, limit_default, limit) = if let Some((desc, limit)) = limit {
            (desc.range.to_string(), desc.default.to_string(), limit.to_string())
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

pub fn print_vfp<I: Iterator<Item=(usize, VfPoint)>>(vfp: I, lock: Option<Microvolts>, core: Option<Microvolts>) {
    let mut table = Table::new();
    table.set_format(table_format());
    table.set_titles(row!["VFP", "Voltage", "Frequency", "Offset", "Default"]);

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
