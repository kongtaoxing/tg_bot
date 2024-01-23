use teloxide::{prelude::*, types::Me, utils::command::BotCommands};
use std::error::Error;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text
    Help,
    /// Start
    Start,
    /// get token/coin price
    P { name: String },
    /// Calculate total price
    Calc { param: String }
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
                bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
            }
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                bot.send_message(msg.chat.id, "输入`/help`查看所有命令。").await?;
            }

            Ok(Command::P { name }) => {
                bot.send_message(msg.chat.id, name.to_string()).await?;
            }

            Ok(Command::Calc { param }) => {
                let cmds: Vec<&str> = param.split_whitespace().collect();
                if cmds.len() != 2 {
                    bot.send_message(msg.chat.id, "参数错误，请按照`/calc [数量] [币名]`输入。").await?;
                    return Ok(());
                }
                else {
                    let num = cmds[0].parse::<i32>();
                    let num = match num {
                        Ok(num) => num,
                        Err(_) => {
                            bot.send_message(msg.chat.id, "参数错误，请按照`/calc [数量] [币名]`输入。").await?;
                            return Ok(());
                        }
                    };
                    let name = cmds[1].to_string();
                    bot.send_message(msg.chat.id, format!("{num} {name}", num=num, name=name)).await?;
                }
                // bot.send_message(msg.chat.id, format!("{num} {name}")).await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}