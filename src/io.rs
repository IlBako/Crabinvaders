#[allow(unused_variables)]
pub trait IOHandler {
    fn read_port(&self, port: u8) -> u8 {
        // Default implementation returns 0 for unhandled ports
        0
    }

    fn write_port(&mut self, port: u8, value: u8) {
        // Default does nothing
    }
}

/// Generic representation of physical arcade controls
#[derive(Debug, Clone, Copy, Default)]
pub struct ArcadeInputs {
    // Shared controls
    pub coin: bool,
    pub tilt: bool,

    // Player 1
    pub p1_start: bool,
    pub p1_left: bool,
    pub p1_right: bool,
    pub p1_fire: bool,

    // Player 2
    pub p2_start: bool,
    pub p2_left: bool,
    pub p2_right: bool,
    pub p2_fire: bool,
}

/// Cabinet configuration switches
#[derive(Debug, Clone, Copy)]
pub struct DipSwitches {
    /// Number of starting lives
    pub starting_lives: StartingLives,
    /// Score threshold for extra lives
    pub extra_ship_threshold: ExtraShipThreshold,
    /// Flag to display or hide the coin insertion prompt on the bottom screen.
    pub display_coin_info: bool,
}

impl Default for DipSwitches {
    fn default() -> Self {
        Self {
            starting_lives: StartingLives::Three,
            extra_ship_threshold: ExtraShipThreshold::At1500Points,
            display_coin_info: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StartingLives {
    Three = 0b00,
    Four = 0b01,
    Five = 0b10,
    Six = 0b11,
}

#[derive(Debug, Clone, Copy)]
pub enum ExtraShipThreshold {
    At1500Points = 0,
    At1000Points = 1,
}
