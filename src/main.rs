#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use cortex_m_rt::entry;

use defmt::println;
use defmt_rtt as _;

use panic_probe as _;
use stm32_hal2::{
    clocks::Clocks,
    gpio::{Pin, PinMode, Port},
    pac,
};

use embedded_alloc::TlsfHeap;

#[global_allocator]
static HEAP: TlsfHeap = TlsfHeap::empty();

const HEAP_SIZE: usize = 16 * 1024;

#[unsafe(link_section = ".uninit")]
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

extern crate alloc;

use kyrylscript::{
    BOOLEAN_TYPE, Constant, INT_TYPE, Instruction, KsCall, NativeHelper, VM, VMResult,
};

use alloc::{boxed::Box, vec};

struct SetLed8 {
    led8: Pin,
}

impl KsCall for SetLed8 {
    fn call<'a>(&mut self, arguments: usize, helper: NativeHelper<'a>) -> VMResult<()> {
        if arguments != 1 {
            return Ok(());
        }

        let gvs = helper.gvs;
        let variable = helper.runner.acc.last(gvs)?;

        if variable.value_type != BOOLEAN_TYPE {
            return Ok(());
        }

        if variable.as_boolean() {
            self.led8.set_high();
            println!("THE LED IS HIGH");
        } else {
            self.led8.set_low();
            println!("THE LED IS LOW");
        }

        Ok(())
    }
}

struct KsDelay {
    delay: Delay,
}

impl KsCall for KsDelay {
    fn call<'a>(&mut self, arguments: usize, helper: NativeHelper<'a>) -> VMResult<()> {
        if arguments != 1 {
            return Ok(());
        }

        let gvs = helper.gvs;
        let variable = helper.runner.acc.last(gvs)?;

        if variable.value_type != INT_TYPE {
            return Ok(());
        }

        let ms = variable.value as u32;
        println!("Waiting");
        self.delay.delay_ms(ms);

        Ok(())
    }
}

#[entry]
fn main() -> ! {
    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    println!("The system has been started");

    let clock_cfg = Clocks::default();
    clock_cfg.setup().unwrap();

    let delay = Delay::new(cp.SYST, clock_cfg.systick());
    let led = Pin::new(Port::A, 8, PinMode::Output);

    let instructions = vec![
        Instruction::LoadConst(Constant::Boolean(false)),
        Instruction::CallNative(0, 1), // Set led 8 to high
        Instruction::ClearAcc,         // clear the stack
        Instruction::LoadConst(Constant::Integer(1_000)),
        Instruction::CallNative(1, 1), // delay for 1_000 ms
        Instruction::ClearAcc,         // clear the stack
        Instruction::LoadConst(Constant::Boolean(true)),
        Instruction::CallNative(0, 1), // Set led 8 to high
        Instruction::ClearAcc,         // clear the stack
        Instruction::LoadConst(Constant::Integer(1_000)),
        Instruction::CallNative(1, 1), // call delay for 1_000 ms
        Instruction::ClearAcc,         // clear the stack
        Instruction::Jump(-12),
    ];

    let mut vm = VM::from(instructions);
    vm.init();
    vm.add_native(Box::new(SetLed8 { led8: led }));
    vm.add_native(Box::new(KsDelay { delay }));

    loop {
        let res = vm.step();

        if let Err(err) = res {
            println!("ERROR: {}", &err.message.as_str());
        }
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
