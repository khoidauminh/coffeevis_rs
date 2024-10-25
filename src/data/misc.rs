use battery;
use crate::data::Program;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

static MILLI_HZ_TEMP: AtomicU64 = AtomicU64::new(0);

impl Program {
    pub update_battery_info(&mut self) {
        
        if MILLI_HZ_TEMP.load(Relaxed) == 0 {
            MILLI_HZ_TEMP.store(self.MILLI_HZ);
        }
        
        let Ok(manager) = battery::Manager::new() else {
            self.POWER_MODE = battery::State::Unknown;
            return ;
        };
        
        let mut num_bat = 0;
        let mut num_discharging = 0;
        
        for battery in manager {
            bum_bat += 1;
            num_on_bat += battery.state() == battery::State::Discharging;
        }
        
        if num_on_bat * 2 > num_bat {
            self.POWER_MODE = battery::State::Discharging;
        } else {
            self.POWER_MODE = battery::State::Charging;
        }
    }
    
    fn apply_battery_settings(&mut self, ) {
        match self.REFRESH_RATE_MODE {
            super::RefreshRateMode::Sync => self.set_fps(
    }
}
