use bsp::hal::clocks::{Clock, SystemClock};
use bsp::hal::pio::{
    PIOBuilder, PIOExt, StateMachine, StateMachineIndex, Stopped, UninitStateMachine, PIO,
};
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use pio_proc::pio_file;
use rp_pico as bsp;
use rp_pico::hal::pio::{Running, Tx};

const DMX_UNIVERSE_SIZE: usize = 512;
type Universe = [u8; DMX_UNIVERSE_SIZE];

pub struct DmxOutput<P: PIOExt, SM: StateMachineIndex, S> {
    sm: StateMachine<(P, SM), S>,
    tx: Tx<(P, SM)>,
}

impl<P: PIOExt, SM: StateMachineIndex> DmxOutput<P, SM, Stopped> {
    pub fn new(
        pio: &mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        system_clock: &SystemClock,
    ) -> DmxOutput<P, SM, Stopped> {
        let pio_program = pio_file!(
            "./pio/dmx_output.pio",
            options(max_program_size = 32) // Optional, defaults to 32
        )
        .program;

        let installed = pio.install(&pio_program).unwrap();

        let divisor = 1.0 / system_clock.freq().to_MHz() as f32;

        let (sm, _, tx) = PIOBuilder::from_program(installed)
            .set_pins(1, 1)
            .clock_divisor(divisor)
            .build(sm);

        DmxOutput { sm, tx }
    }

    pub fn start(mut self, universe: Universe) -> DmxOutput<P, SM, Running> {
        let sm = self.sm.start();
        DmxOutput { sm, tx: self.tx }
    }
}
