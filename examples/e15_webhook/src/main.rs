//! This example demonstrates how to create and execute a webhook.
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;

#[tokio::main]
async fn main() {
    // We don't need a token if we're only handling webhooks.
    let http = Http::without_token();
    // First, we need to retrieve a webhook.
    //
    // You can use `Webhook::from_id` to retrieve a webhook using only its ID, if you have an
    // authenticated HTTP connection. In this example, we could use `Webhook::from_id_with_token`
    // to retrieve the webhook with only its ID and a token, but we'll use a webhook URL instead.
    //
    // You will need to replace the URL with a valid webhook URL for this example to work.
    let webhook = Webhook::from_url(&http, "https://discord.com/api/webhooks/133742013374206969/hello-there-oPNtRN5UY5DVmBe7m1N0HE-replace-me-Dw9LRkgq3zI7LoW3Rb-k-q")
        .await
        .expect("Replace the webhook with your own");

    // We use the `ExecuteWebhook` builder to create the webhook content.
    let builder = ExecuteWebhook::new().content("hello there").username("Webhook test");
    // And finally, we execute the webhook.
    webhook.execute(&http, false, builder).await.expect("Could not execute webhook");
}
