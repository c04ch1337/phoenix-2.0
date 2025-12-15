use std::process::Command;
use thirtyfour::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct DigitalTwin {
    driver: Option<WebDriver>,
}

impl DigitalTwin {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444/wd/hub", caps).await?;
        Ok(Self { driver: Some(driver) })
    }

    // Full file system
    pub fn read_any_file(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    pub fn list_all_files(&self) -> Vec<PathBuf> {
        WalkDir::new("/").into_iter().filter_map(|e| e.ok().map(|e| e.path().to_owned())).collect()
    }

    // App control
    pub fn open_app(&self, app: &str) {
        Command::new(app).spawn().unwrap();
    }

    // Full browser control
    pub async fn goto(&mut self, url: &str) -> Result<(), WebDriverError> {
        // thirtyfour renamed `SessionHandle::get` -> `goto`.
        self.driver.as_ref().unwrap().goto(url).await
    }

    pub async fn login(&mut self, username: &str, password: &str, selector_map: &std::collections::HashMap<&str, &str>) -> Result<(), WebDriverError> {
        let d = self.driver.as_ref().unwrap();
        d.find(By::Id(selector_map["user"])).await?.send_keys(username).await?;
        d.find(By::Id(selector_map["pass"])).await?.send_keys(password).await?;
        d.find(By::Css(selector_map["submit"])).await?.click().await?;
        Ok(())
    }

    pub async fn scrape(&mut self, selector: &str) -> Result<String, WebDriverError> {
        Ok(self.driver.as_ref().unwrap().find(By::Css(selector)).await?.text().await?)
    }

    // Always-aware mode
    pub async fn continuous_mirror(&self) {
        loop {
            // Screenshot + voice + emotion + file changes → feed into context
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }
}
