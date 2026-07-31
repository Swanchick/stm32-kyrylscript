use cortex_m::delay::Delay;

use defmt::println;

use kyrylscript::{INT_TYPE, KsCall, NativeHelper, VMResult};

pub struct KsDelay {
    pub delay: Delay,
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
