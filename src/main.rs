use dotenvy::dotenv;
use twitchchat::PrivmsgExt as _;
use twitchchat::{
    messages::{Commands, Privmsg},
    runner::{AsyncRunner, NotifyHandle, Status},
    UserConfig,
};

mod include;
use crate::include::{channels_to_join, get_user_config};

use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let user_config = get_user_config()?;
    let channels = channels_to_join()?;

    let start = std::time::Instant::now();

    let mut bot = Bot::default()
        .with_command("!hello", |args: Args| {
            let output = format!("hello {}!", args.msg.name());

            args.writer.reply(args.msg, &output).unwrap();
        })
        .with_command("!bottime", move |args: Args| {
            let output = format!("its been running for {:.2?}", start.elapsed());

            args.writer.say(args.msg, &output).unwrap();
        })
        .with_command("!quit", move |args: Args| {
            if args.msg.is_broadcaster() {
                smol::block_on(async move { args.quit.notify().await });
            } else {
                let output = String::from("You are not permitted to do that");
                args.writer.reply(args.msg, &output).unwrap();
            }
        });

    smol::block_on(async move { bot.run(&user_config, &channels).await })
}

struct Args<'a, 'b: 'a> {
    msg: &'a Privmsg<'b>,
    writer: &'a mut twitchchat::Writer,
    quit: NotifyHandle,
}

trait Command: Send + Sync {
    fn handle(&mut self, args: Args<'_, '_>);
}

impl<F> Command for F
where
    F: Fn(Args<'_, '_>),
    F: Send + Sync,
{
    fn handle(&mut self, args: Args<'_, '_>) {
        (self)(args)
    }
}

#[derive(Default)]
struct Bot {
    commands: HashMap<String, Box<dyn Command>>,
}

impl Bot {
    fn with_command(mut self, name: impl Into<String>, cmd: impl Command + 'static) -> Self {
        self.commands.insert(name.into(), Box::new(cmd));
        self
    }

    async fn run(&mut self, user_config: &UserConfig, channels: &[String]) -> anyhow::Result<()> {
        let connector = twitchchat::connector::smol::Connector::twitch()?;

        let mut runner = AsyncRunner::connect(connector, user_config).await?;
        println!("connecting, we are: {}", runner.identity.username());

        for channel in channels {
            println!("joining: {}", channel);
            if let Err(err) = runner.join(channel).await {
                eprintln!("error while joining '{}': {}", channel, err);
            }
        }

        println!("starting main loop");
        self.main_loop(&mut runner).await
    }

    async fn main_loop(&mut self, runner: &mut AsyncRunner) -> anyhow::Result<()> {
        let mut writer = runner.writer();
        let quit = runner.quit_handle();

        loop {
            match runner.next_message().await? {
                Status::Message(Commands::Privmsg(pm)) => {
                    if let Some(cmd) = Self::parse_command(pm.data()) {
                        if let Some(command) = self.commands.get_mut(cmd) {
                            println!("dispatching to: {}", cmd.escape_debug());

                            let args = Args {
                                msg: &pm,
                                writer: &mut writer,
                                quit: quit.clone(),
                            };

                            command.handle(args);
                        }
                    }
                }
                Status::Quit | Status::Eof => break,
                Status::Message(..) => continue,
            }
        }

        println!("end of main loop");
        Ok(())
    }

    fn parse_command(input: &str) -> Option<&str> {
        if !input.starts_with('!') {
            return None;
        }
        input.splitn(2, ' ').next()
    }
}
