use bsp::hal::pio::{
    PIOBuilder, PIOExt, Running, StateMachine, StateMachineIndex, Stopped, UninitStateMachine,
    ValidStateMachine, PIO,
};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use pio_proc::pio_file;
use rp_pico as bsp;

pub struct DmxOutput<P: PIOExt, SM: StateMachineIndex, S> {
    state_machine: StateMachine<(P, SM), S>,
}

impl<P: PIOExt, SM: StateMachineIndex, S> DmxOutput<P, SM, S> {
    pub fn new(
        pio: &mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        syst_clock_mhz: f32,
    ) -> DmxOutput<P, SM, Stopped> {
        let pio_program = pio_file!(
            "./pio/dmx_output.pio",
            options(max_program_size = 32) // Optional, defaults to 32
        )
        .program;

        let installed = pio.install(&pio_program).unwrap();

        let (sm, _, tx) = PIOBuilder::from_program(installed)
            .set_pins(1, 1)
            .clock_divisor(1.0 / syst_clock_mhz)
            .build(sm);

        DmxOutput { state_machine: sm }
    }
}
