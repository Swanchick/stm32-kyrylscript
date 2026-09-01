use alloc::string::{String, ToString};
use defmt::println;
use kyrylscript::{FLOAT_TYPE, INT_TYPE, KsCall, NativeHelper, STRING_TYPE, VMResult};

pub struct KsPrintln;

impl KsCall for KsPrintln {
    fn call<'a>(&mut self, arguments: usize, helper: NativeHelper<'a>) -> VMResult<()> {
        let gvs = helper.gvs;
        let mut output = String::new();

        for _ in 0..arguments {
            let argument = helper.runner.acc.last(gvs)?.clone();

            match argument.value_type {
                INT_TYPE => output.push_str(&(argument.value as i64).to_string()),
                FLOAT_TYPE => output.push_str(&(f64::from_bits(argument.value)).to_string()),
                STRING_TYPE => output.push_str(gvs.collection_string(argument.value as u32)?),
                _ => {}
            }

            helper.runner.acc.pop_data()?;
        }

        println!("{}", &output);

        Ok(())
    }
}
