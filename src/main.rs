#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

use cortex_m::{
    interrupt::{Mutex, free},
    peripheral::NVIC,
};
use cortex_m_rt::entry;

use defmt::println;
use defmt_rtt as _;

use panic_probe as _;
use stm32_hal2::{
    clocks::Clocks,
    gpio::{Pin, PinMode, Port},
    pac::{self, Interrupt, interrupt},
    usart::{Usart, UsartConfig, UsartInterrupt},
};

use embedded_alloc::TlsfHeap;

#[global_allocator]
static HEAP: TlsfHeap = TlsfHeap::empty();

const HEAP_SIZE: usize = 16 * 1024;

#[unsafe(link_section = ".uninit")]
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

extern crate alloc;

mod ks_std;
mod programator;
mod programator_states;

use kyrylscript::{Program, VM};

use crate::{programator::Programator, programator_states::ProgramatorStates};

static PROGRAMATOR: Mutex<RefCell<Programator>> = Mutex::new(RefCell::new(Programator::new()));
static READY: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

    let _cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    println!("The system has been started");

    let clock_cfg = Clocks::default();
    clock_cfg.setup().unwrap();

    let _uart_tx = Pin::new(Port::C, 4, PinMode::Alt(7));
    let _uart_rx = Pin::new(Port::C, 5, PinMode::Alt(7));

    let mut uart = Usart::new(dp.USART1, 115_200, UsartConfig::default(), &clock_cfg).unwrap();
    uart.enable_interrupt(UsartInterrupt::ReadNotEmpty).unwrap();

    unsafe {
        NVIC::unmask(Interrupt::USART1);
    }

    free(|cs| {
        let mut programator = PROGRAMATOR.borrow(cs).borrow_mut();
        programator.uart = Some(uart);
    });

    let mut vm: Option<VM> = None;

    loop {
        let uart_ready = READY.load(Ordering::Relaxed);
        if uart_ready {
            free(|cs| {
                READY.store(false, Ordering::Relaxed);
                let mut programator = PROGRAMATOR.borrow(cs).borrow_mut();
                let bytes = programator.take_bytes();
                let program = Program::deserialize(bytes);
                if let Ok(program) = program {
                    vm = Some(VM::from(program))
                } else {
                    println!("Cannot load the program");
                }
            })
        } else {
            if let Some(vm) = vm.as_mut() {
                if !vm.is_empty() {
                    let _ = vm.step();
                }
            }
        }
    }
}

#[interrupt]
fn USART1() {
    free(|cs| {
        let mut programator = PROGRAMATOR.borrow(cs).borrow_mut();
        let result = programator.load_byte();

        if let Err(error) = result {
            println!("ERROR: {}", error);
            return;
        }

        if let ProgramatorStates::Loaded = programator.state {
            READY.store(true, Ordering::Relaxed);
        }
    });
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
