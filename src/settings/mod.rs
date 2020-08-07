#[path = "../errors/mod.rs"]
mod errors;
use errors::CmdError;
pub struct Settings {
    pub single_word: Option<String>,
    pub path: String,
    pub iterations: usize,
}
impl Default for Settings{
    fn default() -> Settings{
        Settings{single_word: None,path :String::from("words.txt"), iterations : 1_000_000}
    }
}

pub fn settings_from_args() -> Result<Settings, Box<dyn std::error::Error>> {
    let args = &mut std::env::args();
    let size = args.into_iter().len();
    let mut settings: Settings = Default::default();
    if size > 1 {
        if let Some(_) = args.next() {
            while let Some(val) = args.next() {
                let mval: &str = &val.to_string();
                match mval {
                    "-i" => {
                        if let Some(val) = args.next() {
                            settings.path = val.to_owned();
                        } else {
                            return Err(Box::new(CmdError::InputDefault));
                        }
                    }
                    "-I" => {
                        if let Some(val) = args.next() {
                            if let Ok(num_of_iter) = val.parse::<usize>() {
                                settings.iterations = num_of_iter
                            } else {
                                return Err(Box::new(CmdError::IterWrongArgumentType));
                            }
                        } else {
                            return Err(Box::new(CmdError::IterDefault));
                        }
                    }
                    "-w" => {
                        if let Some(val) = args.next() {
                            settings.single_word = Some(val);
                        } else {
                            return Err(Box::new(CmdError::SingleWordDefault));
                        }
                    }
                    _ => {
                        return Err(Box::new(CmdError::NoCmd));
                    }
                }
            }
        }
    }
    Ok(settings)
}
