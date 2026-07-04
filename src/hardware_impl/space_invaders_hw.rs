use crate::io::*;

pub struct SpaceInvadersHardware {
    // Input state
    port1_state: u8,
    port2_state: u8,

    // Configuration settings
    dip_switches: DipSwitches,

    // Shift register state
    shift_register: u16,
    shift_offset: u8,
}

impl SpaceInvadersHardware {
    pub fn new(dip_switches: DipSwitches) -> Self {
        Self {
            port1_state: 0,
            port2_state: 0,
            dip_switches,
            shift_register: 0,
            shift_offset: 0,
        }
    }

    pub fn update_input(&mut self, inputs: &ArcadeInputs) {
        // --- PORT 1 ---
        // Bit 0: Coin (1 = active)
        // Bit 1: P2 Start (1 = active)
        // Bit 2: P1 Start (1 = active)
        // Bit 3: Always 1 (or connected to standard cabinet test buttons)
        // Bit 4: P1 Fire (1 = active)
        // Bit 5: P1 Left (1 = active)
        // Bit 6: P1 Right (1 = active)
        // Bit 7: Unused (Always 0)

        let mut p1: u8 = 0b0000_1000;
        if inputs.coin {
            p1 |= 1 << 0;
        }
        if inputs.p2_start {
            p1 |= 1 << 1;
        }
        if inputs.p1_start {
            p1 |= 1 << 2;
        }
        if inputs.p1_fire {
            p1 |= 1 << 4;
        }
        if inputs.p1_left {
            p1 |= 1 << 5;
        }
        if inputs.p1_right {
            p1 |= 1 << 6;
        }

        self.port1_state = p1;

        // --- PORT 2 ---
        // Bit 0, 1: Starting Lives (00 = 3, 01 = 4, 10 = 5, 11 = 6)
        // Bit 2: Tilt (1 = active)
        // Bit 3: Extra life score (0 = 1500 pts, 1 = 1000 pts)
        // Bit 4: P2 Fire (1 = active)
        // Bit 5: P2 Left (1 = active)
        // Bit 6: P2 Right (1 = active)
        // Bit 7: Display Coin Info on screen (0 = Yes, 1 = No)

        let mut p2 = 0x00;

        // Apply dip switches
        p2 |= self.dip_switches.starting_lives as u8;
        p2 |= (self.dip_switches.extra_ship_threshold as u8) << 3;
        if !self.dip_switches.display_coin_info {
            p2 |= 1 << 7;
        }

        // Apply inputs
        if inputs.tilt {
            p2 |= 1 << 2;
        }
        if inputs.p2_fire {
            p2 |= 1 << 4;
        }
        if inputs.p2_left {
            p2 |= 1 << 5;
        }
        if inputs.p2_right {
            p2 |= 1 << 6;
        }

        self.port2_state = p2;
    }
}

impl IOHandler for SpaceInvadersHardware {
    fn read_port(&self, port: u8) -> u8 {
        match port {
            1 => self.port1_state,
            2 => self.port2_state,
            3 => {
                // Extract the 8-bit window from the 16-bit register
                (self.shift_register >> (8 - self.shift_offset)) as u8
            }
            _ => {
                // Floating bus behavior
                0
            }
        }
    }

    fn write_port(&mut self, port: u8, value: u8) {
        match port {
            2 => {
                // The offset is strictly 0-7, so we mask out higher bits
                self.shift_offset = value & 0x07;
            }
            4 => {
                // Shift the old upper 8 bits to the lower 8 bits
                // Then place the new value in the upper 8 bits
                self.shift_register = (self.shift_register >> 8) | ((value as u16) << 8);
            }
            3 | 5 => {
                // Unimplemented sound triggers
                // unimplemented!();
            }
            _ => {} // Ignore writes to unmapped ports
        }
    }
}
