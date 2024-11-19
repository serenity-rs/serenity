use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;

#[tokio::main]
async fn main() {
    // You don't need a token when you are only dealing with webhooks.
    let http = Http::without_token();
    let webhook = Webhook::from_url(&http, "https://discord.com/api/webhooks/133742013374206969/hello-there-oPNtRN5UY5DVmBe7m1N0HE-replace-me-Dw9LRkgq3zI7LoW3Rb-k-q")
        .await
        .expect("Replace the webhook with your own");

    let builder = ExecuteWebhook::new().content("hello there").username("Webhook test");
    webhook.execute(&http, false, builder).await.expect("Could not execute webhook.");
}
