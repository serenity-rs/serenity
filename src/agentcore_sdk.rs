let voice = reqwest::get("https://main-agentcore.fly.dev/gen?voice=grandma").await?.text().await?;
println!("Grandma says: {}", voice);
