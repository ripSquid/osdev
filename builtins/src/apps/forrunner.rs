use core::str::from_utf8;

use alloc::{string::{String}, boxed::Box};
use base::{LittleManApp, ProgramError};
use fs::{PathString, AppConstructor, DefaultInstall};



#[derive(Default)]
pub struct ForRunner;

impl AppConstructor for ForRunner {
    fn instantiate(&self) -> Box<dyn LittleManApp> {
        Box::new(Self)
    }
}
impl DefaultInstall for ForRunner {
    fn path() -> PathString {
        PathString::from("forrunner.run")
    }
}
impl LittleManApp for ForRunner {
    fn run(&mut self, machine: &mut base::forth::ForthMachine) -> Result<(), ProgramError> {
        let script = {
            let path = machine.stack.try_pop::<String>().ok_or(ProgramError::InvalidStartParameter)?;
            let file = fs::get_file(PathString::from(path)).map_err(|_| ProgramError::FileSystemError)?.read_file().map_err(|_| ProgramError::FileSystemError)?;
            file
        };
        machine.add_instructions_to_end(&from_utf8(&script).map_err(|_| ProgramError::FileSystemError)?);
        machine.run_to_end();
        Ok(())
    }
}