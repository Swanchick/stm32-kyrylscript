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
    pac::{self, Interrupt, USART1, interrupt},
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

use alloc::{boxed::Box, vec::Vec};
use ks_std::{KsDelay, SetLed8};
use kyrylscript::VM;

static BYTECODE: Mutex<RefCell<Vec<u8>>> = Mutex::new(RefCell::new(Vec::new()));
static READY: AtomicBool = AtomicBool::new(false);
static UART: Mutex<RefCell<Option<Usart<USART1>>>> = Mutex::new(RefCell::new(None));

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
        UART.borrow(cs).replace(Some(uart));
    });

    loop {
        let uart_ready = READY.load(Ordering::Relaxed);
        if uart_ready {
            // activate VM for KyrylScript
            // and go to critical_section
        }
    }
}

#[interrupt]
fn USART1() {
    free(|cs| {
        let mut uart_ref = UART.borrow(cs).borrow_mut();
        if let Some(uart) = uart_ref.as_mut() {
            uart.clear_interrupt(UsartInterrupt::ReadNotEmpty);
            let mut bytecode = BYTECODE.borrow(cs).borrow_mut();
            let byte = uart.read_one();
            bytecode.push(byte);

            if bytecode.len() >= 1024 {
                READY.store(true, Ordering::Relaxed);
            }
        }
    });
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
