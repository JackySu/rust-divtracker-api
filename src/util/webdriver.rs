use thirtyfour::prelude::*;

pub async fn get_webdriver() -> WebDriverResult<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();

    let _ = caps.set_disable_web_security();
    let _ = caps.add_chrome_arg("--ssl-protocol=any");
    let _ = caps.add_chrome_arg("--ignore-ssl-errors=true");
    let _ = caps.add_chrome_arg("--disable-extensions");
    let _ = caps.add_chrome_arg("start-maximized");
    let _ = caps.add_chrome_arg("window-size=1280,720");
    let _ = caps.add_chrome_arg("disable-infobars");
    let _ = caps.add_chrome_option("detach", true);

    let port = std::env::var("CHROME_PORT").unwrap_or("9515".to_string());
    let driver = WebDriver::new(format!("{}{}", "http://localhost:", port).as_str(), caps).await?;
    Ok(driver)
}