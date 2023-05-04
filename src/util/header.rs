use reqwest::header;

pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36";
pub static CONTENT_TYPE: &str = "application/json; charset=utf-8";
pub static ACCEPT: &str = "application/json";
pub static REQUEST_PLATFORM_TYPE: &str = "uplay";
pub static REQUEST_WITH: &str = "XMLHttpRequest";
pub static CACHE_CONTROL: &str = "no-cache";
pub static LOCALE: &str = "en-US";
pub static REFERER: &str = "https://connect.ubisoft.com";
pub static HOST: &str = "public-ubiservices.ubi.com";
pub static ENCODING: &str = "gzip, deflate, br";
pub static UBI_LOCALE_CODE: &str = "en-US";
pub static UBI_APPID: &str = "314d4fef-e568-454a-ae06-43e3bece12a6";
pub async fn get_common_header() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE.parse().unwrap());
    headers.insert(header::USER_AGENT, USER_AGENT.parse().unwrap());
    headers.insert(header::ACCEPT, ACCEPT.parse().unwrap());
    headers.insert(header::HOST, HOST.parse().unwrap());
    headers.insert(header::CACHE_CONTROL, CACHE_CONTROL.parse().unwrap());
    headers.insert(header::ACCEPT_LANGUAGE, LOCALE.parse().unwrap());
    headers.insert(header::ACCEPT_ENCODING, ENCODING.parse().unwrap());
    headers.insert(header::REFERER, REFERER.parse().unwrap());
    headers.insert(header::ORIGIN, REFERER.parse().unwrap());
    headers.insert("Ubi-AppId", UBI_APPID.parse().unwrap());
    headers.insert("Ubi-RequestedPlatformType", REQUEST_PLATFORM_TYPE.parse().unwrap());
    headers.insert("Ubi-LocaleCode", UBI_LOCALE_CODE.parse().unwrap());
    headers.insert("X-Requested-With", REQUEST_WITH.parse().unwrap());
    headers
}