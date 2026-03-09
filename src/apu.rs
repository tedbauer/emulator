// Game Boy APU — 4-channel audio processing unit.
//
// Channels:
//   1  Square wave with frequency sweep  (0xFF10–0xFF14)
//   2  Square wave                       (0xFF16–0xFF19)
//   3  Wave table                        (0xFF1A–0xFF1E, wave RAM 0xFF30–0xFF3F)
//   4  Noise / LFSR                      (0xFF20–0xFF23)
//
// Control: NR50/NR51/NR52               (0xFF24–0xFF26)

#![allow(dead_code)]

const CPU_FREQ: u32 = 4_194_304;
pub const SAMPLE_RATE: u32 = 44_100;

// Each u8 bit-pattern encodes 8 waveform steps from MSB→LSB.
// A set bit means the channel output is HIGH for that step.
const DUTY_WAVEFORMS: [u8; 4] = [
    0b00000001, // 12.5 %
    0b10000001, // 25 %
    0b10000111, // 50 %
    0b01111110, // 75 %
];

// Channel 4 noise frequency divisors indexed by NR43 bits 2-0.
const NOISE_DIVISORS: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

// ---------------------------------------------------------------------------
// Channel 1 / 2 — square wave
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct SquareChannel {
    // Registers
    duty: u8,
    volume: u8,
    env_initial: u8,
    env_add: bool,
    env_period: u8,
    env_timer: u8,
    freq: u16, // 11-bit
    length_enable: bool,
    // Sweep (channel 1 only)
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    // Runtime
    enabled: bool,
    dac_enabled: bool,
    timer: i32,
    phase_step: u8, // 0..7
    length_counter: u16,
}

impl SquareChannel {
    fn trigger(&mut self, has_sweep: bool) {
        self.enabled = self.dac_enabled;
        self.timer = 4 * (2048 - self.freq as i32);
        self.phase_step = 0;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.volume = self.env_initial;
        self.env_timer = self.env_period;
        if has_sweep {
            self.sweep_timer = if self.sweep_period > 0 {
                self.sweep_period
            } else {
                8
            };
            self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;
            if self.sweep_shift > 0 {
                self.do_sweep_overflow_check();
            }
        }
    }

    fn calc_sweep_freq(&self) -> u16 {
        let delta = self.freq >> self.sweep_shift;
        if self.sweep_negate {
            self.freq.wrapping_sub(delta)
        } else {
            self.freq.wrapping_add(delta)
        }
    }

    fn do_sweep_overflow_check(&mut self) {
        if self.calc_sweep_freq() > 2047 {
            self.enabled = false;
        }
    }

    fn tick_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period > 0 {
                self.sweep_period
            } else {
                8
            };
            if self.sweep_enabled && self.sweep_period > 0 {
                let new_freq = self.calc_sweep_freq();
                if new_freq > 2047 {
                    self.enabled = false;
                } else {
                    self.freq = new_freq;
                    self.do_sweep_overflow_check();
                }
            }
        }
    }

    fn tick_length(&mut self) {
        if self.length_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    fn tick_volume(&mut self) {
        if self.env_period == 0 {
            return;
        }
        if self.env_timer > 0 {
            self.env_timer -= 1;
        }
        if self.env_timer == 0 {
            self.env_timer = self.env_period;
            if self.env_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    /// Advance by `cycles` T-cycles; return raw amplitude 0-15.
    fn step(&mut self, cycles: u32) -> u8 {
        if !self.enabled {
            return 0;
        }
        self.timer -= cycles as i32;
        while self.timer <= 0 {
            self.timer += 4 * (2048 - self.freq as i32);
            self.phase_step = (self.phase_step + 1) & 7;
        }
        let high = DUTY_WAVEFORMS[self.duty as usize] & (0x80 >> self.phase_step) != 0;
        if high {
            self.volume
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Channel 3 — wave table
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct WaveChannel {
    dac_enabled: bool,
    output_level: u8, // 0 mute, 1 full, 2 half, 3 quarter
    freq: u16,
    length_enable: bool,
    enabled: bool,
    timer: i32,
    position: u8, // 0..31 within wave RAM
    length_counter: u16,
    wave_ram: [u8; 16], // 32 nibbles packed
}

impl Default for WaveChannel {
    fn default() -> Self {
        WaveChannel {
            dac_enabled: false,
            output_level: 0,
            freq: 0,
            length_enable: false,
            enabled: false,
            timer: 0,
            position: 0,
            length_counter: 0,
            wave_ram: [0u8; 16],
        }
    }
}

impl WaveChannel {
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        self.timer = 2 * (2048 - self.freq as i32);
        self.position = 0;
        if self.length_counter == 0 {
            self.length_counter = 256;
        }
    }

    fn tick_length(&mut self) {
        if self.length_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    fn step(&mut self, cycles: u32) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        self.timer -= cycles as i32;
        while self.timer <= 0 {
            self.timer += 2 * (2048 - self.freq as i32);
            self.position = (self.position + 1) & 31;
        }
        let byte = self.wave_ram[(self.position / 2) as usize];
        let nibble = if self.position & 1 == 0 {
            (byte >> 4) & 0xF
        } else {
            byte & 0xF
        };
        match self.output_level {
            1 => nibble,
            2 => nibble >> 1,
            3 => nibble >> 2,
            _ => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Channel 4 — noise (LFSR)
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct NoiseChannel {
    volume: u8,
    env_initial: u8,
    env_add: bool,
    env_period: u8,
    env_timer: u8,
    clock_shift: u8,
    wide_mode: bool, // true = 7-bit LFSR, false = 15-bit
    divisor_code: u8,
    length_enable: bool,
    enabled: bool,
    dac_enabled: bool,
    timer: i32,
    lfsr: u16,
    length_counter: u16,
}

impl NoiseChannel {
    fn timer_period(&self) -> i32 {
        (NOISE_DIVISORS[self.divisor_code as usize] << self.clock_shift) as i32
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        self.timer = self.timer_period();
        self.lfsr = 0x7FFF;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.volume = self.env_initial;
        self.env_timer = self.env_period;
    }

    fn tick_length(&mut self) {
        if self.length_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    fn tick_volume(&mut self) {
        if self.env_period == 0 {
            return;
        }
        if self.env_timer > 0 {
            self.env_timer -= 1;
        }
        if self.env_timer == 0 {
            self.env_timer = self.env_period;
            if self.env_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    fn step(&mut self, cycles: u32) -> u8 {
        if !self.enabled {
            return 0;
        }
        self.timer -= cycles as i32;
        while self.timer <= 0 {
            self.timer += self.timer_period();
            let xor = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
            self.lfsr >>= 1;
            self.lfsr |= xor << 14;
            if self.wide_mode {
                self.lfsr = (self.lfsr & !(1 << 6)) | (xor << 6);
            }
        }
        // LFSR bit 0 LOW means channel is outputting HIGH
        if self.lfsr & 1 == 0 {
            self.volume
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// APU
// ---------------------------------------------------------------------------

pub struct Apu {
    ch1: SquareChannel,
    ch2: SquareChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    nr50: u8,
    nr51: u8,
    powered: bool,

    // Frame sequencer: clocked at 512 Hz (every 8192 T-cycles)
    frame_seq_timer: u32,
    frame_seq_step: u8,

    // Sample-rate tracking: emit a sample every CPU_FREQ / SAMPLE_RATE T-cycles
    sample_accum: u32,

    // DC blocker state (simple high-pass to remove DC offset)
    dc_prev_in_l: i32,
    dc_prev_out_l: i32,
    dc_prev_in_r: i32,
    dc_prev_out_r: i32,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            ch1: SquareChannel::default(),
            ch2: SquareChannel::default(),
            ch3: WaveChannel::default(),
            ch4: NoiseChannel::default(),
            nr50: 0x77,
            nr51: 0xF3,
            powered: true,
            frame_seq_timer: 8192,
            frame_seq_step: 0,
            sample_accum: 0,
            dc_prev_in_l: 0,
            dc_prev_out_l: 0,
            dc_prev_in_r: 0,
            dc_prev_out_r: 0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        // All writes except NR52 are ignored when powered off
        if !self.powered && addr != 0xFF26 {
            return;
        }
        match addr {
            // Channel 1
            0xFF10 => {
                self.ch1.sweep_period = (val >> 4) & 0x7;
                self.ch1.sweep_negate = val & 0x08 != 0;
                self.ch1.sweep_shift = val & 0x07;
            }
            0xFF11 => {
                self.ch1.duty = (val >> 6) & 0x3;
                self.ch1.length_counter = 64 - (val & 0x3F) as u16;
            }
            0xFF12 => {
                self.ch1.env_initial = (val >> 4) & 0xF;
                self.ch1.env_add = val & 0x08 != 0;
                self.ch1.env_period = val & 0x07;
                self.ch1.dac_enabled = (val & 0xF8) != 0;
                if !self.ch1.dac_enabled {
                    self.ch1.enabled = false;
                }
            }
            0xFF13 => {
                self.ch1.freq = (self.ch1.freq & 0x700) | val as u16;
            }
            0xFF14 => {
                self.ch1.freq = (self.ch1.freq & 0xFF) | (((val & 0x07) as u16) << 8);
                self.ch1.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch1.trigger(true);
                }
            }
            0xFF15 => {} // NR20 — unused
            // Channel 2
            0xFF16 => {
                self.ch2.duty = (val >> 6) & 0x3;
                self.ch2.length_counter = 64 - (val & 0x3F) as u16;
            }
            0xFF17 => {
                self.ch2.env_initial = (val >> 4) & 0xF;
                self.ch2.env_add = val & 0x08 != 0;
                self.ch2.env_period = val & 0x07;
                self.ch2.dac_enabled = (val & 0xF8) != 0;
                if !self.ch2.dac_enabled {
                    self.ch2.enabled = false;
                }
            }
            0xFF18 => {
                self.ch2.freq = (self.ch2.freq & 0x700) | val as u16;
            }
            0xFF19 => {
                self.ch2.freq = (self.ch2.freq & 0xFF) | (((val & 0x07) as u16) << 8);
                self.ch2.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch2.trigger(false);
                }
            }
            // Channel 3
            0xFF1A => {
                self.ch3.dac_enabled = val & 0x80 != 0;
                if !self.ch3.dac_enabled {
                    self.ch3.enabled = false;
                }
            }
            0xFF1B => {
                self.ch3.length_counter = 256 - val as u16;
            }
            0xFF1C => {
                self.ch3.output_level = (val >> 5) & 0x3;
            }
            0xFF1D => {
                self.ch3.freq = (self.ch3.freq & 0x700) | val as u16;
            }
            0xFF1E => {
                self.ch3.freq = (self.ch3.freq & 0xFF) | (((val & 0x07) as u16) << 8);
                self.ch3.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch3.trigger();
                }
            }
            0xFF1F => {} // NR40 — unused
            // Channel 4
            0xFF20 => {
                self.ch4.length_counter = 64 - (val & 0x3F) as u16;
            }
            0xFF21 => {
                self.ch4.env_initial = (val >> 4) & 0xF;
                self.ch4.env_add = val & 0x08 != 0;
                self.ch4.env_period = val & 0x07;
                self.ch4.dac_enabled = (val & 0xF8) != 0;
                if !self.ch4.dac_enabled {
                    self.ch4.enabled = false;
                }
            }
            0xFF22 => {
                self.ch4.clock_shift = (val >> 4) & 0xF;
                self.ch4.wide_mode = val & 0x08 != 0;
                self.ch4.divisor_code = val & 0x07;
            }
            0xFF23 => {
                self.ch4.length_enable = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch4.trigger();
                }
            }
            // Control
            0xFF24 => self.nr50 = val,
            0xFF25 => self.nr51 = val,
            0xFF26 => {
                let was_on = self.powered;
                self.powered = val & 0x80 != 0;
                if was_on && !self.powered {
                    self.ch1 = SquareChannel::default();
                    self.ch2 = SquareChannel::default();
                    self.ch3 = WaveChannel::default();
                    self.ch4 = NoiseChannel::default();
                    self.nr50 = 0;
                    self.nr51 = 0;
                }
            }
            // Wave RAM
            0xFF30..=0xFF3F => {
                self.ch3.wave_ram[(addr - 0xFF30) as usize] = val;
            }
            _ => {}
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => {
                0x80 | (self.ch1.sweep_period << 4)
                    | ((self.ch1.sweep_negate as u8) << 3)
                    | self.ch1.sweep_shift
            }
            0xFF11 => 0x3F | (self.ch1.duty << 6),
            0xFF12 => {
                (self.ch1.env_initial << 4) | ((self.ch1.env_add as u8) << 3) | self.ch1.env_period
            }
            0xFF13 => 0xFF,
            0xFF14 => 0xBF | ((self.ch1.length_enable as u8) << 6),
            0xFF15 => 0xFF,
            0xFF16 => 0x3F | (self.ch2.duty << 6),
            0xFF17 => {
                (self.ch2.env_initial << 4) | ((self.ch2.env_add as u8) << 3) | self.ch2.env_period
            }
            0xFF18 => 0xFF,
            0xFF19 => 0xBF | ((self.ch2.length_enable as u8) << 6),
            0xFF1A => 0x7F | ((self.ch3.dac_enabled as u8) << 7),
            0xFF1B => 0xFF,
            0xFF1C => 0x9F | (self.ch3.output_level << 5),
            0xFF1D => 0xFF,
            0xFF1E => 0xBF | ((self.ch3.length_enable as u8) << 6),
            0xFF1F => 0xFF,
            0xFF20 => 0xFF,
            0xFF21 => {
                (self.ch4.env_initial << 4) | ((self.ch4.env_add as u8) << 3) | self.ch4.env_period
            }
            0xFF22 => {
                (self.ch4.clock_shift << 4)
                    | ((self.ch4.wide_mode as u8) << 3)
                    | self.ch4.divisor_code
            }
            0xFF23 => 0xBF | ((self.ch4.length_enable as u8) << 6),
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let ch = (self.ch4.enabled as u8) << 3
                    | (self.ch3.enabled as u8) << 2
                    | (self.ch2.enabled as u8) << 1
                    | (self.ch1.enabled as u8);
                0x70 | ((self.powered as u8) << 7) | ch
            }
            0xFF27..=0xFF2F => 0xFF,
            0xFF30..=0xFF3F => self.ch3.wave_ram[(addr - 0xFF30) as usize],
            _ => 0xFF,
        }
    }

    /// Advance by `cycles` T-cycles. Returns `Some((left, right))` when a sample is ready.
    pub fn tick(&mut self, cycles: u32) -> Option<(i16, i16)> {
        if !self.powered {
            self.sample_accum += cycles * SAMPLE_RATE;
            if self.sample_accum >= CPU_FREQ {
                self.sample_accum -= CPU_FREQ;
                return Some((0, 0));
            }
            return None;
        }

        // Advance frame sequencer (512 Hz = every 8192 T-cycles)
        if self.frame_seq_timer > cycles {
            self.frame_seq_timer -= cycles;
        } else {
            self.frame_seq_timer += 8192 - cycles;
            self.tick_frame_sequencer();
        }

        // Step each channel
        let s1 = self.ch1.step(cycles);
        let s2 = self.ch2.step(cycles);
        let s3 = self.ch3.step(cycles);
        let s4 = self.ch4.step(cycles);

        // Check if a sample is due
        self.sample_accum += cycles * SAMPLE_RATE;
        if self.sample_accum < CPU_FREQ {
            return None;
        }
        self.sample_accum -= CPU_FREQ;

        // Mix channels per NR51 panning.
        // NR51: bit7=ch4L 6=ch3L 5=ch2L 4=ch1L | 3=ch4R 2=ch3R 1=ch2R 0=ch1R
        let mut raw_l: i32 = 0;
        let mut raw_r: i32 = 0;
        if self.nr51 & 0x10 != 0 {
            raw_l += s1 as i32;
        }
        if self.nr51 & 0x01 != 0 {
            raw_r += s1 as i32;
        }
        if self.nr51 & 0x20 != 0 {
            raw_l += s2 as i32;
        }
        if self.nr51 & 0x02 != 0 {
            raw_r += s2 as i32;
        }
        if self.nr51 & 0x40 != 0 {
            raw_l += s3 as i32;
        }
        if self.nr51 & 0x04 != 0 {
            raw_r += s3 as i32;
        }
        if self.nr51 & 0x80 != 0 {
            raw_l += s4 as i32;
        }
        if self.nr51 & 0x08 != 0 {
            raw_r += s4 as i32;
        }

        // Apply master volume (NR50 bits 6-4 = left, 2-0 = right, values 0-7 → 1-8)
        let vol_l = (((self.nr50 >> 4) & 0x7) as i32) + 1;
        let vol_r = ((self.nr50 & 0x7) as i32) + 1;
        let mixed_l = raw_l * vol_l; // 0..480
        let mixed_r = raw_r * vol_r;

        // DC-block filter: y[n] = x[n] - x[n-1] + (255/256) * y[n-1]
        // Removes the constant positive bias so silence → 0.
        let out_l = mixed_l - self.dc_prev_in_l + ((self.dc_prev_out_l * 255) >> 8);
        let out_r = mixed_r - self.dc_prev_in_r + ((self.dc_prev_out_r * 255) >> 8);
        self.dc_prev_in_l = mixed_l;
        self.dc_prev_out_l = out_l;
        self.dc_prev_in_r = mixed_r;
        self.dc_prev_out_r = out_r;

        // Scale to i16: max DC-blocked swing ≈ ±480, scale by 68 ≈ ±32640
        let l = (out_l * 68).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let r = (out_r * 68).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        Some((l, r))
    }

    fn tick_frame_sequencer(&mut self) {
        match self.frame_seq_step {
            // Even steps clock length counters
            0 | 4 => {
                self.ch1.tick_length();
                self.ch2.tick_length();
                self.ch3.tick_length();
                self.ch4.tick_length();
            }
            // Steps 2 and 6 also clock frequency sweep
            2 | 6 => {
                self.ch1.tick_length();
                self.ch2.tick_length();
                self.ch3.tick_length();
                self.ch4.tick_length();
                self.ch1.tick_sweep();
            }
            // Step 7 clocks volume envelopes
            7 => {
                self.ch1.tick_volume();
                self.ch2.tick_volume();
                self.ch4.tick_volume();
            }
            _ => {}
        }
        self.frame_seq_step = (self.frame_seq_step + 1) & 7;
    }
}
