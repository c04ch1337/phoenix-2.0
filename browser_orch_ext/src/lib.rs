use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

// Represents the browser process
pub struct Browser {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Browser {
    // Launch a new browser instance with a persistent user profile
    pub fn launch(user_data_dir: &str) -> Result<Self, std::io::Error> {
        let mut process = Command::new("node")
            .arg("browser_orch_ext/main.js")
            .arg(user_data_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().expect("Failed to open stdin");
        let stdout = BufReader::new(process.stdout.take().expect("Failed to open stdout"));

        let mut browser = Self { process, stdin, stdout };

        // Wait for the "READY" signal from the Node.js script
        let mut line = String::new();
        browser.stdout.read_line(&mut line)?;
        if !line.starts_with("READY") {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to launch browser"));
        }

        Ok(browser)
    }

    fn send_command(&mut self, command: &str) -> Result<String, std::io::Error> {
        self.stdin.write_all(command.as_bytes())?;
        self.stdin.write_all(b"\n")?;

        let mut line = String::new();
        self.stdout.read_line(&mut line)?;

        if line.starts_with("SUCCESS") {
            Ok(line.strip_prefix("SUCCESS ").unwrap_or("").trim().to_string())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, line))
        }
    }

    // Go to a specific URL
    pub fn goto(&mut self, url: &str) -> Result<(), std::io::Error> {
        self.send_command(&format!("GOTO {}", url))?;
        Ok(())
    }

    // Execute arbitrary JavaScript on the page
    pub fn execute_js(&mut self, js_code: &str) -> Result<String, std::io::Error> {
        self.send_command(&format!("EXECUTE_JS {}", js_code))
    }

    // Close the browser
    pub fn close(&mut self) -> Result<(), std::io::Error> {
        self.send_command("CLOSE")?;
        self.process.wait()?;
        Ok(())
    }
}
