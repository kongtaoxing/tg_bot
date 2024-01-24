use teloxide::{prelude::*, types::{Me, ParseMode::MarkdownV2}, utils::command::BotCommands};
use std::{env, error::Error};
use reqwest::{header::{HeaderMap, HeaderValue, USER_AGENT}, Response};
use serde_json::Value;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Start
    Start,
    /// Display this text
    Help,
    /// get token/coin price
    P { name: String },
    /// Calculate total price
    Calc { param: String },
    /// information
    Info
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                let mut description = String::new();
                description.push_str(format!("/start - 启动机器人\n").as_str());
                description.push_str(format!("/help - 查看所有命令\n").as_str());
                description.push_str(format!("/p [币名] - 获取币价\n").as_str());
                description.push_str(format!("/calc [数量] [币名] - 计算总价\n").as_str());
                description.push_str(format!("/info - 机器人信息\n").as_str());
                println!("description: {:?}", description);
                bot.send_message(msg.chat.id, description.to_string()).await?;
            }
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                bot.send_message(msg.chat.id, "输入 `/help` 查看所有命令。\n\n本 bot 开发者为 [Coulson](tg://user?id=1481722371)")
                    .parse_mode(MarkdownV2)
                    .await?;
            }

            Ok(Command::P { name }) => {
                let price = get_price(1.0, name.clone()).await?;
                bot.send_message(msg.chat.id, price.to_string())
                .parse_mode(MarkdownV2)
                .await?;
            }

            Ok(Command::Calc { param }) => {
                let cmds: Vec<&str> = param.split_whitespace().collect();
                if cmds.len() != 2 {
                    bot.send_message(msg.chat.id, format!("参数错误，请按照`/calc [数量] [币名]`输入。\n\n示例：`/calc 10 BTC`"))
                        .parse_mode(MarkdownV2)
                        .await?;
                    return Ok(());
                }
                else {
                    let amount = cmds[0].parse::<f32>();
                    let amount = match amount {
                        Ok(amount) => amount,
                        Err(_) => {
                            bot.send_message(msg.chat.id, "参数错误，请按照 /calc [数量] [币名] 输入。\n\n示例：`/calc 10 BTC`")
                                .parse_mode(MarkdownV2)
                                .await?;
                            return Ok(());
                        }
                    };
                    let name = cmds[1].to_string();
                    let data = get_price(amount, name.clone()).await?;
                    println!("data: {:?}", data);

                    bot.send_message(msg.chat.id, format!("`{amount}` 个\n{data}"))
                        .parse_mode(MarkdownV2)
                        .await?;
                }
            }

            Ok(Command::Info) => {
                bot.send_message(msg.chat.id, "本 bot 能够实时计算特定数量的币种价格。\n\n价格 api 来自 [CoinmarketCap](https://coinmarketcap.com/)\n\n开发者为 [Coulson](tg://user?id=1481722371)")
                    .parse_mode(MarkdownV2)
                    .await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

pub async fn get_price(amount: f32, name: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://pro-api.coinmarketcap.com/v2/tools/price-conversion?symbol={name}&amount={amount}",
        name = name,
        amount = amount
    );
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(
        "X-CMC_PRO_API_KEY",
        HeaderValue::from_str(&env::var("CMC_API_KEY").unwrap()).unwrap(),
    );

    let client = reqwest::Client::new();
    let res: Response = client.get(&url).headers(headers).send().await?;
    
    println!("res status: {:?}", res.status());

    if res.status().is_success() {
        let price_json_raw = res.text().await?;
        let price_json: Value = serde_json::from_str(&price_json_raw).unwrap();
        println!("price_json: {:?}", price_json["data"]);
        let mut data = String::new();
        for obj in price_json["data"].as_array().unwrap() {
            let name = obj["name"].to_string().replace("\"", "");
            let symbol = obj["symbol"].to_string().replace("\"", "");
            let price = obj["quote"]["USD"]["price"].to_string();
            let raw_data = format!("\n`{}`（`{}`）的价格为: {} USD\n", name, symbol, price);
            data.push_str(&raw_data);
        }
        Ok(data.to_string().replace(".", "\\."))
    } else if res.status().as_u16() == 400 {
        let price_json_raw = res.text().await?;
        let price_json: Value = serde_json::from_str(&price_json_raw).unwrap();
        println!("price_json: {:?}", price_json["status"]["error_message"]);
        Ok(price_json["status"]["error_message"].to_string().replace("\\", "").replace("\"", ""))
    } else {
        println!("error: {:?}", res.status());
        Ok(String::from("CoinMarketCap API error, please try again later."))
    }
}