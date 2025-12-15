use anyhow::Result;
use headless_chrome::Browser;

/// The handle to a chromium process.
pub struct ChromiumProcess {
    #[allow(dead_code)]
    browser: Browser,
}

impl ChromiumProcess {
    pub fn new() -> Result<Self> {
        let browser = Browser::default()?;
        Ok(Self { browser })
    }
}
