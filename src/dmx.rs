use bsp::hal::clocks::{Clock, SystemClock};
use bsp::hal::pio::{PIOBuilder, PIOExt, StateMachine, StateMachineIndex, UninitStateMachine, PIO};
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use pio_proc::pio_file;
use rp_pico as bsp;
use rp_pico::hal::pio::{Running, Tx};

/// The required clock frequency for the DMX PIO program to work
const PIO_CLOCK_MHZ: usize = 1;

const DMX_UNIVERSE_SIZE: usize = 512;
type Universe = [u8; DMX_UNIVERSE_SIZE];

pub struct DmxPIO<P: PIOExt, SM: StateMachineIndex> {
    sm: StateMachine<(P, SM), Running>,
    tx: Tx<(P, SM)>,
}

impl<P: PIOExt, SM: StateMachineIndex> DmxPIO<P, SM> {
    pub fn new(
        pio: &mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        system_clock: &SystemClock,
    ) -> Option<DmxPIO<P, SM>> {
        let pio_program = pio_file!(
            "./pio/dmx_output.pio",
            options(max_program_size = 32) // Optional, defaults to 32
        )
        .program;

        let installed = pio.install(&pio_program).ok()?;
        debug!("Installed program....");

        let divisor = PIO_CLOCK_MHZ as f32 / system_clock.freq().to_MHz() as f32;
        debug!("Clock divisor set to: {=f32}", divisor);

        let (sm_stopped, _, tx) = PIOBuilder::from_program(installed)
            .set_pins(1, 1)
            .clock_divisor(divisor)
            .build(sm);

        let sm = sm_stopped.start();
        Some(DmxPIO { sm, tx })
    }

    pub fn start_frame(&mut self) {
        self.sm.restart();
    }

    pub fn push_channel(&mut self, channel: u8) -> bool {
        while self.tx.is_full() {
            cortex_m::asm::nop();
        }
        self.tx.write(channel as u32)
    }

    pub fn drain(&mut self) {
        self.tx.drain_fifo();
    }
}

pub struct Dmx<P: PIOExt, SM: StateMachineIndex> {
    pio: DmxPIO<P, SM>,
}

impl<P: PIOExt, SM: StateMachineIndex> Dmx<P, SM> {
    /// Create a new DMX instance. returns [None] if there is no more program space in the PIO
    pub fn new(pio: DmxPIO<P, SM>) -> Dmx<P, SM> {
        Dmx { pio }
    }

    pub fn send_universe(&mut self, universe: &[u8]) {
        self.pio.start_frame();
        for channel in universe {
            self.pio.push_channel(*channel);
        }
        self.pio.drain();
    }
}

pub struct DmxAsync<P: PIOExt, SM: StateMachineIndex> {
    pio: DmxPIO<P, SM>,
    dma_channel: usize,
    buffer: Universe,
}
impl<P: PIOExt, SM: StateMachineIndex> Dmx<P, SM> {}
