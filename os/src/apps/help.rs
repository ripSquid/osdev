use alloc::boxed::Box;

use crate::fs::{Path, AppConstructor};

use super::{KaggApp, InstallableApp};

pub struct HelpApp(Language);
pub struct Help;
enum Language {
    Swedish,
    English,
}
impl AppConstructor for Help {
    fn instantiate(&self) -> Box<dyn KaggApp> {
        Box::new(HelpApp(Language::Swedish))
    }
}
impl InstallableApp for Help {
    fn install() -> (Path, Box<dyn AppConstructor>) {
        (Path::from("help"), Box::new(Self))
    }
}
impl KaggApp for HelpApp {
    fn start(&mut self, args: &[&str]) -> Result<(), super::StartError> {
        match args.get(0) {
            Some(&"eng") => self.0 = Language::English,
            Some(&"swe") => self.0 = Language::Swedish,
            Some(_) |
            None => (),
        }
        Ok(())
    }
    fn update(&mut self, handle: &mut super::KaggHandle) {
        let text = match &self.0 {
            Language::Swedish => 
            "Välkommen till ett gymnasiearbete gjort av två elever på Lars Kagg Skolan.
            Detta operativsystem kommer med olika demon och verktyg som visar vad det är kapabelt av.
            För att lista alla program och filer, skriv 'dir' 
            
            For english help, write 'help eng'",
            Language::English => "Welcome to a Thesis work created by two students at the school of Lars Kagg.
            This operating system comes with demos and tools which displays its full capabilities.
            To list all programs and files, write 'dir'",
        };
        if let Ok(formatter) = handle.text_mode_formatter() {
            formatter.next_line().write_str(text);
        }
        handle.call_exit();
    }
}