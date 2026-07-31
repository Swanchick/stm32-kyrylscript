use defmt::println;

use stm32_hal2::gpio::Pin;

use kyrylscript::{BOOLEAN_TYPE, KsCall, NativeHelper, VMResult};

pub struct SetLed8 {
    pub led8: Pin,
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
