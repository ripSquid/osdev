use core::ops::{Deref, DerefMut};

use alloc::{vec::Vec, string::{String, ToString}};

use crate::{forth::{Stack, ForthMachine}, display::{VgaModeSwitch, DefaultVgaWriter, BitmapVgaWriter}, input::{Keyboard, KEYBOARD_QUEUE}};

pub trait LittleManApp: Send + Sync + 'static {
    fn start(&mut self, _args: &mut Stack) -> Result<(), StartError> {
        Ok(())
    }
    fn update(&mut self, handle: &mut OsHandle);
    fn shutdown(&mut self) {}
}

#[derive(Debug)]
pub enum StartError {}
pub struct OsHandle {
    fm: Option<*mut ForthMachine>,
    control_flow: ControlFlow,
    graphics: GraphicsHandle,
    calls: Vec<SystemCall>
}
pub enum SystemCall {
    SwitchGraphics(VgaModeSwitch),
    ForthFunction(String),
}
impl OsHandle {
    pub fn keyboard(&self) -> &Keyboard<char> {
        unsafe { &KEYBOARD_QUEUE }
    }
    pub fn execute(&mut self, forth_command: impl ToString) -> bool {
        match self.fm.map(|s|unsafe{s.as_mut()}).flatten() {
            Some(accessible) => {
                accessible.run(forth_command.to_string(), self.text_mode_formatter().unwrap());
                true
            },
            None => {
                self.calls.push(SystemCall::ForthFunction(forth_command.to_string()));
                false
            },
        }
    }
    pub fn keyboard_mut(&mut self) -> &mut Keyboard<char> {
        unsafe {&mut KEYBOARD_QUEUE}
    }
    pub fn running(&self) -> bool {
        self.control_flow == ControlFlow::Running
    }
    pub fn new(formatter: impl Into<GraphicsHandle>) -> Self {
        Self { control_flow: ControlFlow::Running, graphics: formatter.into(), calls: Vec::new(), fm: None}
    }
    pub unsafe fn new_complicated(formatter: impl Into<GraphicsHandle>, machine: &mut ForthMachine) -> Self {
        
        Self { control_flow: ControlFlow::Running, graphics: formatter.into(), calls: Vec::new(), fm: Some(machine as *mut _)}
    }
    pub fn flush_calls(&mut self) -> Vec<SystemCall> {
        self.calls.split_off(0)
    }
    pub fn call_exit(&mut self) {
        self.control_flow = ControlFlow::Quit;
    }
    pub fn text_mode_formatter(&mut self) -> Result<&mut DefaultVgaWriter, VideoModeError> {
        if let GraphicsHandleType::TextMode(formatter) = &mut *self.graphics {
            Ok(unsafe {&mut **formatter})
        } else {
            Err(VideoModeError::IsInGraphicsMode)
        }
    }
}
#[derive(Debug)]
pub enum VideoModeError {
    IsInGraphicsMode
}
pub struct GraphicsHandle {
    formatter: GraphicsHandleType,
}
impl Into<GraphicsHandle> for GraphicsHandleType {
    fn into(self) -> GraphicsHandle {
        GraphicsHandle { formatter: self }
    }
}

impl Deref for GraphicsHandle {
    type Target = GraphicsHandleType;

    fn deref(&self) -> &Self::Target {
        &self.formatter
    }
}
impl DerefMut for GraphicsHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.formatter
    }
}
pub enum GraphicsHandleType {
    TextMode(*mut DefaultVgaWriter),
    GraphicsMode(BitmapVgaWriter),
}
#[derive(PartialEq)]
pub enum ControlFlow {
    Running,
    Quit,
}