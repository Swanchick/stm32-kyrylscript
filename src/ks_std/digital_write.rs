use stm32_hal2::gpio::{Pin, PinMode, Port};

use kyrylscript::{BOOLEAN_TYPE, INT_TYPE, KsCall, NativeHelper, STRING_TYPE, VMError, VMResult};

pub struct DigitalWrite;

impl KsCall for DigitalWrite {
    fn call<'a>(&mut self, arguments: usize, helper: NativeHelper<'a>) -> VMResult<()> {
        if arguments != 3 {
            return Ok(());
        }

        let gvs = helper.gvs;
        let runner = helper.runner;
        let port = runner.acc.last(gvs)?.clone();
        if port.value_type != STRING_TYPE {
            return Err(VMError::from("Variable is not a string"));
        }
        let port = gvs.collection_string(port.value as u32)?;

        runner.acc.pop_data()?;
        let port = match port {
            "A" => Ok(Port::A),
            "B" => Ok(Port::B),
            "C" => Ok(Port::C),
            _ => Err("Invalid port line"),
        }?;

        let pin = runner.acc.pop(gvs)?;
        if pin.value_type != INT_TYPE {
            return Err(VMError::from("Variable is not an int"));
        }
        let mut pin = Pin::new(port, pin.value as u8, PinMode::Output);

        let toggle = runner.acc.pop(gvs)?;
        if toggle.value_type != BOOLEAN_TYPE {
            return Err(VMError::from("Variable is not boolean"));
        }

        if toggle.as_boolean() {
            pin.set_high();
        } else {
            pin.set_low();
        }

        Ok(())
    }
}
