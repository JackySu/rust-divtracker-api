# rust-divtracker-api

> 本项目使用http请求库 [reqwest](https://docs.rs/reqwest/latest/reqwest/) 访问育碧官方api来获取 全境封锁1 玩家数据
> 
> 使用 selenium 库 [thirtyfour](https://docs.rs/thirtyfour/latest/thirtyfour/) 来模拟浏览器环境从 api.tracker.gg 获取全境封锁2玩家数据

## 安装

0. 在目录下新建 .env 或手动设置环境变量
  
  ```
  DATABASE_URL=sqlite://{数据库路径}
  UBI_USERNAME={育碧账户邮箱}
  UBI_PASSWORD={育碧账户密码}
  CHROME_PORT={chromedriver.exe监听端口，默认9515}
  ```

1. 运行:
  
  ```
  cargo run -r
  ```

## 工作原理

0. 使用 [Rocket](https://rocket.rs/) 响应请求

1. 如何登录并访问育碧api？

    <!> 注意 以下 **大部分** http 请求都将用到这些 `公共请求头`
    ```
    Accept: application/json, text/plain, */*
    User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36
    Content-Type: application/json; charset=utf-8
    Host: public-ubiservices.ubi.com
    Cache-Control: no-cache
    Accept-Language: en-US
    Accept-Encoding: gzip, deflate, br
    Referer: https://connect.ubisoft.com
    Origin: https://connect.ubisoft.com
    Ubi-AppId: 314d4fef-e568-454a-ae06-43e3bece12a6
    Ubi-RequestedPlatformType: uplay
    Ubi-LocaleCode: en-US
    X-Requested-With: XMLHttpRequest
    ```
  
    1. 使用 `公共请求头` 外加 

       ```
       Authorization: {base64加密(育碧账户邮箱+':'+育碧账户密码)}
       ```

       POST `https://public-ubiservices.ubi.com/v3/profiles/sessions`

       成功则获取 `ticket` 和 `sessionId`

    2. 使用 `公共请求头` 外加 

       ```
       Authorization: Ubi_v1 t={ticket}
       Ubi-SessionId: {sessionId}
       ```

       GET `https://public-ubiservices.ubi.com/v2/profiles?nameOnPlatform={你想查找的名字}&platformType=uplay`

       成功则获取 `profileId`

    3. 使用和以上 `2` 相同的请求头

       GET `https://public-ubiservices.ubi.com/v1/profiles/{profileId}/statscard?spaceId={游戏spaceId}`

       成功则获取玩家游戏数据

2. 如何访问 tracker.gg 的 api ？

    1. 使用 selenium 模拟浏览器环境直接访问 https://api.tracker.gg/api/v2/division-2/standard/profile/uplay/{玩家名}

## 育碧 我是你爹

**..i..**
