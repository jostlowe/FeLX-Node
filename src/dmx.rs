use bsp::hal::clocks::{Clock, SystemClock};
use bsp::hal::pio::{PIOBuilder, PIOExt, StateMachine, StateMachineIndex, UninitStateMachine, PIO};
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use pio_proc::pio_file;
use rp_pico as bsp;
use rp_pico::hal::{
    gpio::{bank0::BankPinId, FunctionPio0, Pin, PinId},
    pio::{Running, Tx},
};
use rp_pico::pac::DMA;

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
        pin: u8,
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
            .set_pins(pin, 1)
            .side_set_pin_base(pin)
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

pub struct Dmx<P, SM, N>
where
    P: PIOExt,
    SM: StateMachineIndex,
    N: PinId + BankPinId,
{
    pio: DmxPIO<P, SM>,
    pin: Pin<N, FunctionPio0>,
}

impl<P, SM, N> Dmx<P, SM, N>
where
    P: PIOExt,
    SM: StateMachineIndex,
    N: PinId + BankPinId,
{
    pub fn new(pio: DmxPIO<P, SM>, pin: Pin<N, FunctionPio0>) -> Dmx<P, SM, N> {
        Dmx { pio, pin }
    }

    pub fn send_universe(&mut self, universe: &[u8]) {
        self.pio.start_frame();
        for channel in universe {
            self.pio.push_channel(*channel);
        }
        self.pio.drain();
    }
}

/*
pub struct DmxAsync<P, SM, N>
where
    P: PIOExt,
    SM: StateMachineIndex,
    N: PinId + BankPinId,
{
    pio: DmxPIO<P, SM>,
    pin: Pin<N, FunctionPio0>,
    dma_channel: usize,
    pub buffer: Universe,
}

impl<P, SM, N> DmxAsync<P, SM, N>
where
    P: PIOExt,
    SM: StateMachineIndex,
    N: PinId + BankPinId,
{
    pub fn new(
        pio: DmxPIO<P, SM>,
        pin: Pin<N, FunctionPio0>,
        dma_channel: usize,
    ) -> DmxAsync<P, SM, N> {
        DmxAsync {
            pio,
            pin,
            dma_channel,
            buffer: [0; DMX_UNIVERSE_SIZE],
        }
    }

    pub fn start(&mut self, dma: &mut DMA) {
        dma.ch[self.dma_channel]
            .ch_ctrl_trig
            .modify(|_, w| w.en().set_bit().data_size().size_byte().treq_sel());

        self.pio.start_frame();
    }
}
*/
