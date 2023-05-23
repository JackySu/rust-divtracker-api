use thirtyfour::prelude::*;

pub async fn get_webdriver() -> WebDriverResult<WebDriver> {
    let caps = DesiredCapabilities::chrome();
    let port = std::env::var("CHROME_PORT").unwrap_or("9515".to_string());
    let driver = WebDriver::new(format!("{}{}", "http://localhost:", port).as_str(), caps).await?;
    Ok(driver)
}