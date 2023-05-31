use crate::command::Command;

pub struct SpeedCalculatorInvoker {
    pub command: Option<Box<dyn Command>>,
}

impl SpeedCalculatorInvoker {
    pub fn set_command(&mut self, command: Box<dyn Command>) {
        self.command = Some(command);
    }

    pub fn execute_command(&self) {
        if let Some(command) = &self.command {
            command.execute();
        }
    }
}
