use std::time::Duration;
use std::thread::sleep;
use std::{io, iter};
use nvapi::{
    Gpu, ClockDomain,
    CoolerPolicy, CoolerLevel,
    Microvolts, Kilohertz, KilohertzDelta, Percentage, Range,
};
use nvapi::nvapi::{
    ClockFrequencyType,
};
use Error;

pub struct AutoDetectOptions {
    pub fan_override: bool,
    pub step: KilohertzDelta,
    pub test: Option<String>,
    pub voltage_wait_delay: Duration,
    pub max_frequency: Kilohertz,
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

            let current_frequency = self.gpu.inner().clock_frequencies(ClockFrequencyType::Current)?
                .get(&ClockDomain::Graphics).cloned().ok_or("couldn't read GPU clock")?;

            if current_frequency == frequency {
                warn!("{} @ {} probably means flat VFP line", current_frequency, current_voltage);
                break
            }

            let sec = Duration::from_secs(1);
            sleep(sec);
            delay -= sec;
        }

        Ok(false)
    }

    pub fn test_prepare(&self) -> Result<(), Error> {
        if !self.options.fan_override {
            self.gpu.set_cooler_levels(vec![CoolerLevel {
                policy: CoolerPolicy::Manual,
                level: Percentage(85),
            }].into_iter())?
        }

        let info = self.gpu.info()?;

        self.gpu.set_power_limits(info.power_limits.iter().map(|info| info.range.max))?;
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
        self.gpu.set_vfp_lock(voltage)?;
        let reached_voltage = if !self.wait_for_voltage(voltage, frequency, self.options.voltage_wait_delay)? {
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
            Ok(true)
        }?;

        if reached_voltage {
            let clock = self.current_clock()?;
            if clock != frequency {
                warn!("Clock throttle detected, expected {} but got {}", frequency, clock);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn run_test_operation(&mut self, voltage: Microvolts, frequency: Kilohertz) -> Result<bool, Error> {
        if let Some(ref test) = self.options.test {
            unimplemented!()
        } else {
            //unimplemented!()
            loop {
                println!("Stable? (y/n): ");
                let mut s = String::new();
                io::stdin().read_line(&mut s)?;
                match &s[..1] {
                    "y" => return Ok(true),
                    "n" => return Ok(false),
                    _ => (),
                }
            }
        }
    }

    pub fn test_point(&mut self, index: usize, voltage: Microvolts, frequency: Kilohertz, delta: KilohertzDelta) -> Result<Option<(KilohertzDelta, Kilohertz)>, Error> {
        let base_frequency = frequency - delta;

        info!("Testing point {}: current frequency {} (base {})", voltage, frequency, base_frequency);

        if !self.set_voltage(voltage, frequency)? {
            warn!("Skipping {}: failed to set", voltage);
            return Ok(None)
        }

        let mut valid = Range {
            max: if let Some(ref prev) = self.previous_clock {
                *prev - base_frequency
            } else {
                self.options.max_frequency - base_frequency
            },
            min: delta,
        };

        loop {
            let delta = (valid.max - valid.min) * 3 / 4;
            let delta = delta / self.options.step.0 * self.options.step.0;
            let delta = valid.min + delta;
            if delta == valid.min {
                break
            }
            println!("{} delta vs {} range", delta, valid);

            let frequency = base_frequency + delta;
            info!("Testing {}: {}", voltage, frequency);
            self.gpu.set_vfp(iter::once((index, delta)), iter::empty())?;
            let result = self.run_test_operation(voltage, frequency)?;

            if result {
                valid.min = delta;
            } else {
                valid.max = delta - self.options.step;
            }

            println!("range now  {:?}", valid);
            if valid.max <= valid.min {
                break
            }
        }

        let frequency = base_frequency + valid.min;
        self.previous_clock = Some(frequency);
        Ok(Some((valid.min, frequency)))
    }
}
