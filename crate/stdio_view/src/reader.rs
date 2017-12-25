use std::io::{self, Write};
use std::sync::mpsc::Sender;

use console::style;
use console::Term;

/// Name of this reader, useful when naming threads.
pub const NAME: &'static str = concat!(module_path!(), "::StdinReader");

/// Reads lines from stdin and sends them to the [`StdinSystem`](struct.StdinSystem.html).
///
/// This should be run in a separate thread to the system as input from stdin is blocking, and the
/// system needs to be responsive to changes in the ECS `World`.
#[derive(Debug)]
pub struct StdinReader {
    /// Channel sender to the endpoint for input from stdin.
    system_tx: Sender<String>,
}

impl StdinReader {
    /// Returns a StdinReader.
    ///
    /// # Parameters:
    ///
    /// * `system_tx`: Channel sender to the System for input from stdin.
    pub fn new(system_tx: Sender<String>) -> Self {
        StdinReader { system_tx }
    }

    /// Signals this reader to read from stdin.
    pub fn start(&self) {
        let mut term = Term::stdout();
        let prompt = format!("{}: ", style(">>").blue().bold());

        let mut buffer = String::new();
        loop {
            write!(term, "{}", &prompt).expect("Failed to write stdio prompt");
            match io::stdin().read_line(&mut buffer) {
                Ok(n) => {
                    if n > 0 {
                        buffer.truncate(n);
                        if let Err(_) = self.system_tx.send(buffer.trim().to_string()) {
                            // TODO: log
                            break;
                        }
                    }
                }
                Err(e) => panic!("{:?}", e),
            }

            buffer.clear();
        }
    }
}

// TODO: integration test
