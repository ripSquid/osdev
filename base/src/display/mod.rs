mod buffer;
mod formatter;
pub mod macros;
pub mod primitives;
mod universal;
pub use universal::*;
mod vga;
mod vga_graphics;
pub use buffer::*;
pub use formatter::*;
pub use vga::*;
pub use vga_graphics::*;
mod mode_switch;
pub use mode_switch::*;
