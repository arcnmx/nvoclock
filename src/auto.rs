use std::process::Command;
use std::sync::mpsc::{channel, TryRecvError};
use std::time::Duration;
use std::thread::{spawn,sleep};
use std::{io, iter};
use log::{warn, info};
use nvapi::nvapi::PerfFlags;
use nvapi::{
    Gpu, ClockDomain,
    CoolerPolicy, CoolerSettings,
    Microvolts, Kilohertz, KilohertzDelta, Percentage, Range,
    nvapi::ClockFrequencyType,
};
use crate::Error;

pub struct AutoDetectOptions {
    pub fan_override: bool,
    pub voltage_override: bool,
    pub perflimit_override: bool,
    pub step: KilohertzDelta,
    pub test: Option<String>,
    pub voltage_wait_delay: Duration,
}

pub struct AutoDetect<'a> {
    pub gpu: &'a Gpu,
    pub options: AutoDetectOptions,
    pub previous_clock: Option<Kilohertz>,
    pub voltage_boost: Percentage,
    pub range: Range<KilohertzDelta>,
}

impl<'a> AutoDetect<'a> {
    pub fn new(gpu: &'a Gpu, options: AutoDetectOptions) -> Result<Self, Error> {
        Ok(AutoDetect {
            options: options,
            previous_clock: None,
            voltage_boost: gpu.inner().core_voltage_boost()?,
            range: gpu.info()?.vfp_limits.get(&ClockDomain::Graphics).ok_or("couldn't read GPU clock range")?.range,
            gpu: gpu,
        })
    }

    pub fn current_clock(&self) -> Result<Kilohertz, Error> {
        self.gpu.inner().clock_frequencies(ClockFrequencyType::Current)?
            .get(&ClockDomain::Graphics).cloned().ok_or("couldn't read GPU clock".into())
    }

    pub fn wait_for_voltage(&self, voltage: Microvolts, frequency: Kilohertz, mut delay: Duration) -> Result<bool, Error> {
        while delay.as_secs() > 0 {
            let current_voltage = self.gpu.inner().core_voltage()?;
            if current_voltage == voltage {
                return Ok(true)
            }

            let current_frequency = self.current_clock()?;

            if current_frequency >= frequency && current_voltage < voltage {
                warn!("{} is equal or higher than {} with lower voltage, passing", current_frequency, frequency);
                return Ok(true);
            }

            let sec = Duration::from_secs(1);
            sleep(sec);
            delay -= sec;
        }

        Ok(false)
    }

    pub fn test_prepare(&self) -> Result<(), Error> {
        if !self.options.fan_override {
            self.gpu.set_cooler_levels(self.gpu.info()?.coolers.into_iter()
                .map(|(id, _)| (id, CoolerSettings::new(Some(Percentage(85))))),
            )?
        }

        let info = self.gpu.info()?;
        if !self.options.perflimit_override {
            self.gpu.set_power_limits(info.power_limits.iter().map(|info| info.range.max))?;
        }
            //self.gpu.reset_vfp()?;

        Ok(())
    }

    pub fn test_cleanup(&self) -> Result<(), Error> {
        if !self.options.fan_override {
            self.gpu.reset_cooler_levels()?;
        }

        Ok(())
    }

    pub fn set_voltage(&mut self, voltage: Microvolts, frequency: Kilohertz) -> Result<bool, Error> {
        self.gpu.set_vfp_lock_voltage(Some(voltage))?;
        let reached_voltage = if !self.wait_for_voltage(voltage, frequency, self.options.voltage_wait_delay)? {
            if !self.options.voltage_override {
                let full = Percentage(100);
                if self.voltage_boost < full {
                    warn!("Boosting core voltage");
                    self.gpu.set_voltage_boost(full)?;
                    self.voltage_boost = full;
                    self.wait_for_voltage(voltage, frequency, self.options.voltage_wait_delay)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }?;

        Ok(reached_voltage)
    }

    pub fn run_test_operation(&mut self, voltage: Microvolts, frequency: Kilohertz) -> Result<PerfFlags, Error> {
        if let Some(ref test) = self.options.test {
            let (from_test, to_mon) = channel();
            let test_path = self.options.test.clone().unwrap();
            let thr = spawn(move || {
                let result = Command::new(test_path)
                                            .status()
                                            .expect("executable not found")
                                            .success();
                let _ = from_test.send(result);
            });
            let mut perf_limits = self.gpu.status().unwrap().perf.limits;
            let mut fail_count = 0;
            let mut clock_fail_count = 0;
            perf_limits = perf_limits & PerfFlags::NO_LOAD_LIMIT;
            loop {
                if fail_count > 10 {
                    warn!("Too much query fails, considering test failed.");
                    return Ok(PerfFlags::NO_LOAD_LIMIT);
                }
                sleep(Duration::from_secs(1));

                let status = self.gpu.status();
                if status.is_err() {
                    warn!("Failed to get GPU data - {}", status.err().unwrap());
                    fail_count += 1;
                    continue;
                }
                fail_count = 0;
                perf_limits = perf_limits | self.gpu.status().unwrap().perf.limits;

                let clock = self.current_clock()?;
                if (clock < frequency - self.options.step) {
                    if (clock_fail_count < 10) {
                        warn!("Clock throttle detected, expected {} but got {}", frequency, clock);
                        clock_fail_count += 1;
                    }
                }

                let test_res = to_mon.try_recv();
                if test_res.is_ok() {
                    if test_res.unwrap() {
                        let triggered_flags = perf_limits - PerfFlags::NO_LOAD_LIMIT;
                        if (clock_fail_count > 9) {
                            return Ok(triggered_flags);
                        } else {
                            // if gpu manages to keep clocks, then let it do that
                            return Ok(PerfFlags::empty());
                        }
                    } else {
                        return Ok(PerfFlags::NO_LOAD_LIMIT);
                    }
                }
            }
        } else {
            loop {
                println!("Stable? (y/n): ");
                let mut s = String::new();
                io::stdin().read_line(&mut s)?;
                match &s[..1] {
                    "y" => return Ok(PerfFlags::empty()),
                    "n" => return Ok(PerfFlags::NO_LOAD_LIMIT),
                    _ => (),
                }
            }
        }
    }

    pub fn test_point(&mut self, index: usize, voltage: Microvolts, frequency: Kilohertz, mut delta: KilohertzDelta) -> Result<Option<(KilohertzDelta, Kilohertz)>, Error> {
        let base_frequency = frequency - delta;

        info!("Testing point {}: current frequency {} (base {})", voltage, frequency, base_frequency);
        println!("Testing point {}: current frequency {} (base {})", voltage, frequency, base_frequency);

        let clock = self.current_clock()?;
        // if clock - self.options.step*4 > frequency {
        //     while base_frequency + delta < clock {
        //         delta = delta + self.options.step;
        //     }
        //     delta = delta - self.options.step*4;
        //     warn!("Boosting clock to {} to match previous point",base_frequency + delta);
        // }
        
        loop {
            // re-set voltage every time to counter driver resets
            self.gpu.reset_vfp_lock()?;
            if !self.set_voltage(voltage, frequency)? {
                warn!("Skipping {}: failed to set", voltage);
                return Ok(None)
            }
            info!("Testing {}: {}", voltage, base_frequency + delta);
            self.gpu.set_vfp(iter::once((index, delta)), iter::empty())?;
            // find a way to determine if there is a lower voltage point with same freq and it's failing instead
            info!("Current {}: {}", self.gpu.inner().core_voltage()?, self.current_clock()?);
            let result = self.run_test_operation(voltage, self.current_clock()?)?;
            // warn!("{}",result);
            if result == PerfFlags::empty() {
                info!("{} passed test", base_frequency + delta);
                delta = delta + self.options.step;
            } else {
                if result == PerfFlags::NO_LOAD_LIMIT {
                    warn!("{} failed, stepping back 3 times", base_frequency + delta);
                    delta = delta - self.options.step * 3;
                } else {
                    warn!("Performance was limited by {}. Considering this as a limit.", result);
                    delta = delta - self.options.step;
                }
                if delta < KilohertzDelta(0) { delta = KilohertzDelta(0); }
                self.gpu.set_vfp(iter::once((index, delta)), iter::empty())?;
                return Ok(Some((delta, base_frequency + delta)));
            }
        }
    }
}
