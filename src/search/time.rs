use std::time::{Duration, Instant};

use crate::types::{MAX_DEPTH, Side};

#[derive(Debug)]
pub struct TimeManager {
    pub clock: Instant,
    pub settings: TimeSettings,
    pub limits: Limits,
}

#[derive(Debug, Default)]
pub struct TimeSettings {
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub movestogo: usize,
    pub depth: usize,
    pub nodes: usize,
    pub mate: usize,
    pub movetime: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    time: u64,
    depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            time: 300000,
            depth: MAX_DEPTH - 1,
        }
    }
}

impl TimeManager {
    pub fn new() -> TimeManager {
        TimeManager {
            clock: Instant::now(),
            settings: TimeSettings::default(),
            limits: Limits::default(),
        }
    }

    pub fn clear_settings(&mut self) {
        self.settings = TimeSettings::default();
    }

    pub fn reset_clock(&mut self) {
        self.clock = Instant::now();
    }

    pub fn set_time_limit(&mut self, side: Side) {
        let remaining_time;
        let increment;

        match side {
            Side::White => {
                remaining_time = self.settings.wtime;
                increment = self.settings.winc;
            }
            Side::Black => {
                remaining_time = self.settings.btime;
                increment = self.settings.binc;
            }
        }

        if remaining_time == 0 {
            self.limits.time = 300000; //Default
            return;
        }

        self.limits.time = (remaining_time / 20) + (increment / 2); //Simple time managment strategy: remaining time/20 + increment/2
    }

    pub fn set_depth_limit(&mut self) {
        if self.settings.depth == 0 {
            self.limits.depth = MAX_DEPTH - 1; //Default 
            return;
        }

        self.limits.depth = self.settings.depth;
    }

    pub fn depth_limit(&self) -> usize {
        self.limits.depth
    }

    pub fn time_limit(&self) -> u64 {
        self.limits.time
    }

    pub fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    pub fn over_limit(&self) -> bool {
        self.elapsed().as_millis() as u64 > self.limits.time + 20 //So it uses the same time as an older version that didn't stop exactly over the limit
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}
