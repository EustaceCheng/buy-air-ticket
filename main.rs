use prettytable::{row, Table};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT,
};
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use chrono::{Duration, Utc};

const SESSION_ID: &str = "674d65d6c41c38dc5903809c";
const ORIGIN_AIRPORT: &str = "FUK";
const DESTINATION_AIRPORT: &str = "TPE";
const USER_CURRENCY: &str = "TWD";
const PRICING_CURRENCY: &str = "TWD";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Tigerair Results:");

    fetch_and_display_top_10().await?;
    Ok(())
}

async fn fetch_and_display_top_10() -> Result<(), Box<dyn Error>> {
    // 設置 headers
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        ORIGIN,
        HeaderValue::from_static("https://booking.tigerairtw.com"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://booking.tigerairtw.com/"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"));

    // 自定義 headers
    headers.insert("priority", HeaderValue::from_static("u=1, i"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Google Chrome\";v=\"129\", \"Not=A?Brand\";v=\"8\", \"Chromium\";v=\"129\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"macOS\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-site"));
    headers.insert(
        "x-device-id",
        HeaderValue::from_static("9I5fn7QEdlRLUKeNhVqvS"),
    );
    headers.insert("x-language", HeaderValue::from_static("zh-TW"));


    let (since, until) = get_date_range();

    // 設置 body 資料
    let body = json!({
        "operationName": "appLiveDailyPrices",
        "variables": {
            "input": {
                "sessionId": SESSION_ID,
                "origin": ORIGIN_AIRPORT,
                "destination": DESTINATION_AIRPORT,
                "userCurrency": USER_CURRENCY,
                "pricingCurrency": PRICING_CURRENCY,
                "since": since,
                "until": until,
                "source": "resultPagePriceBrick"
            }
        },
        "query": "query appLiveDailyPrices($input: QueryLiveDailyPricesInput!) {\n  appLiveDailyPrices(input: $input) {\n    origin\n    destination\n    date\n    currency\n    amount\n    fareLabels {\n      id\n    }\n  }\n}\n"
    });

    // 發送請求
    let client = reqwest::Client::new();
    let res = client
        .post("https://api-book.tigerairtw.com/graphql")
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    // 解析回應
    let response_text = res.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;

    // 檢查 sessionId 過期的錯誤
    if response_json["errors"].is_array() {
        let error_message = response_json["errors"][0]["message"]
            .as_str()
            .unwrap_or("Unknown error");
        if error_message == "Internal server error" {
            println!("SessionId 過期");
            return Ok(());
        }
    }

    // 提取需要的資料
    let prices = &response_json["data"]["appLiveDailyPrices"];

    // 存儲價格的向量
    let mut price_list: Vec<(String, i64)> = Vec::new();

    for price in prices.as_array().unwrap() {
        let date = price["date"].as_str().unwrap().to_string();
        let amount = price["amount"].as_i64().unwrap();
        price_list.push((date, amount));
    }

    // 對價格進行排序
    price_list.sort_by(|a, b| a.1.cmp(&b.1)); // 根據 amount 進行排序

    // 創建表格
    let mut table = Table::new();
    table.add_row(row!["Date", "Amount"]);

    // 將排序後的價格添加到表格中
    for (date, amount) in price_list.iter().take(10) {
        table.add_row(row![date, amount]);
    }

    // 輸出表格
    table.printstd();
    Ok(())
}


fn get_date_range() -> (String, String) {
    let since = Utc::now().date_naive()+ Duration::days(30);
    let until = since + Duration::days(87);
    (since.to_string(), until.to_string())
}



