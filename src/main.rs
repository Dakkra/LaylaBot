use once_cell::sync::Lazy;
use serenity::all::{GuildId, Member, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;

static USER_MESSAGED_STATE: Lazy<Mutex<HashMap<UserInfo, bool>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let user_id = new_member.user.id;
        let guild_id = new_member.guild_id;
        let user_info = UserInfo { user_id, guild_id };
        tokio::spawn(handle_user_timout(ctx, user_info));
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // If the message is from a server, store that this user messaged in that server
        let guild_id = msg.guild_id;
        if let Some(guild_id) = guild_id {
            let user_info = UserInfo {
                user_id: msg.author.id,
                guild_id,
            };
            println!(
                "User {} messaged in guild {}",
                user_info.user_id, user_info.guild_id
            );
            USER_MESSAGED_STATE.lock().await.insert(user_info, true);
        }

        // Commands
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }
}

async fn handle_user_timout(ctx: Context, info: UserInfo) {
    println!(
        "User {} has joined guild {} and will be evaluated for removal",
        info.user_id, info.guild_id
    );

    // Wait for timeout to eval user
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    println!(
        "User {} timed out and will be evaluated for removal",
        info.user_id
    );

    let did_user_message = USER_MESSAGED_STATE
        .lock()
        .await
        .remove(&info)
        .unwrap_or(false);
    if did_user_message {
        println!(
            "User {} Has messaged since joining and will not be kicked",
            info.user_id
        );
        return;
    }

    let thing = info.guild_id.kick(&ctx, info.user_id).await;
    match thing {
        Ok(_val) => {
            println!(
                "User {} has been kicked form guild {}",
                info.user_id, info.guild_id
            )
        }
        Err(why) => println!("Error kicking user: {why:?}"),
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct UserInfo {
    user_id: UserId,
    guild_id: GuildId,
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("LAYLA_BOT_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
