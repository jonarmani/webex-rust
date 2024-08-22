use std::env;

const BOT_ACCESS_TOKEN: &str = "BOT_ACCESS_TOKEN";
const BOT_EMAIL: &str = "BOT_EMAIL";

///
/// # Autoreply
///
/// This example replies to any message sent directly to the bot, or sent in a space (room)
/// provided the bot is mentioned.
///
/// # Usage
///
/// BOT_ACCESS_TOKEN="<token>" BOT_EMAIL="botname@webex.bot" cargo run --example auto-reply
///
/// You can obtain a bot token by logging into the [Cisco Webex developer site](https://developer.webex.com/), then
///
/// * Select "My Webex Apps" from your profile menu (available by clicking on your avatar on the top right)
/// * Select "Create New App"
/// * Select "Create a Bot"
/// * Choose something unique to yourself for testing, e.g., "username-hello"
/// * **Save** the "Bot's Access Token" you see on the next page.  If you fail to do so, you can
///   regenerate it later, but this will invalidate the old token.
///

#[tokio::main]
async fn main() {
    let token = env::var(BOT_ACCESS_TOKEN)
        .unwrap_or_else(|_| panic!("{} not specified in environment", BOT_ACCESS_TOKEN));
    let bot_email = env::var(BOT_EMAIL)
        .unwrap_or_else(|_| panic!("{} not specified in environment", BOT_EMAIL));

    let webex = webex::Webex::new(token.as_str()).await;
    let mut event_stream = webex.event_stream().await.expect("event stream");

    while let Ok(event) = event_stream.next().await {
        // Dig out the useful bit
        if event.activity_type() == webex::ActivityType::Message(webex::MessageActivity::Posted) {
            // The event stream doesn't contain the message -- you have to go fetch it
            if let Ok(msg) = webex
                .get::<webex::Message>(&event.try_global_id().unwrap())
                .await
            {
                match &msg.person_email {
                    // Reply as long as it doesn't appear to be our own message
                    // In practice, this shouldn't happen since bots can't see messages
                    // that don't specifically mention them (i.e., appears in the special
                    // "mentions" field).
                    Some(sender) if sender != bot_email.as_str() => {
                        let mut reply = webex::types::MessageOut::from(&msg);
                        reply.text = Some(format!("{}, you said: {}", sender, msg.text.unwrap()));
                        webex.send_message(&reply).await.unwrap();
                    }
                    _ => (),
                }
            }
        }
    }
}
