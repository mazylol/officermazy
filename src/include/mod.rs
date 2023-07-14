#![allow(dead_code)]
use anyhow::Context as _;
use twitchchat::{messages, AsyncRunner, Status, UserConfig};

fn get_env_var(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("please set `{}`", key))
}

pub fn get_user_config() -> anyhow::Result<twitchchat::UserConfig> {
    let name = get_env_var("TWITCH_NAME")?;
    let token = get_env_var("TWITCH_TOKEN")?;

    let config = UserConfig::builder()
        .name(name)
        .token(token)
        .enable_all_capabilities()
        .build()?;

    Ok(config)
}

pub fn channels_to_join() -> anyhow::Result<Vec<String>> {
    let channels = get_env_var("TWITCH_CHANNEL")?
        .split(',')
        .map(ToString::to_string)
        .collect();

    Ok(channels)
}

pub async fn main_loop(mut runner: AsyncRunner) -> anyhow::Result<()> {
    loop {
        match runner.next_message().await? {
            Status::Message(msg) => {
                handle_message(msg).await;
            }
            Status::Quit => {
                println!("we signaled we wanted to quit");
                break;
            }
            Status::Eof => {
                println!("we got a 'normal' eof");
                break;
            }
        }
    }

    Ok(())
}

pub async fn handle_message(msg: messages::Commands<'_>) {
    use messages::Commands::*;

    match msg {
        Privmsg(msg) => println!("[{}] {}: {}", msg.channel(), msg.name(), msg.data()),

        Raw(_) => {}

        IrcReady(_) => {}
        Ready(_) => {}
        Cap(_) => {}

        ClearChat(_) => {}
        ClearMsg(_) => {}
        GlobalUserState(_) => {}
        HostTarget(_) => {}
        Join(_) => {}
        Notice(_) => {}
        Part(_) => {}
        Ping(_) => {}
        Pong(_) => {}
        Reconnect(_) => {}
        RoomState(_) => {}
        UserNotice(_) => {}
        UserState(_) => {}
        Whisper(_) => {}

        _ => {}
    }
}
