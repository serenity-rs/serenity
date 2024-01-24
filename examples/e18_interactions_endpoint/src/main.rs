use serenity::builder::*;
use serenity::interactions_endpoint::Verifier;
use serenity::model::application::*;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

fn handle_command(interaction: CommandInteraction) -> CreateInteractionResponse<'static> {
    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!(
        "Hello from interactions webhook HTTP server! <@{}>",
        interaction.user.id
    )))
}

fn handle_request(
    mut request: tiny_http::Request,
    body: &mut Vec<u8>,
    verifier: &Verifier,
) -> Result<(), Error> {
    println!("Received request from {:?}", request.remote_addr());

    // Read the request body (containing the interaction JSON)
    body.clear();
    request.as_reader().read_to_end(body)?;

    // Reject request if it fails cryptographic verification
    // Discord rejects the interaction endpoints URL if this check is not done
    // (This part is very specific to your HTTP server crate of choice, so serenity cannot abstract
    // away the boilerplate)
    let find_header =
        |name| Some(request.headers().iter().find(|h| h.field.equiv(name))?.value.as_str());
    let signature = find_header("X-Signature-Ed25519").ok_or("missing signature header")?;
    let timestamp = find_header("X-Signature-Timestamp").ok_or("missing timestamp header")?;
    if verifier.verify(signature, timestamp, body).is_err() {
        request.respond(tiny_http::Response::empty(401))?;
        return Ok(());
    }

    // Build Discord response
    let response = match serde_json::from_slice::<Interaction>(body)? {
        // Discord rejects the interaction endpoints URL if pings are not acknowledged
        Interaction::Ping(_) => CreateInteractionResponse::Pong,
        Interaction::Command(interaction) => handle_command(interaction),
        _ => return Ok(()),
    };

    // Send the Discord response back via HTTP
    request.respond(
        tiny_http::Response::from_data(serde_json::to_vec(&response)?)
            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap()),
    )?;

    Ok(())
}

fn main() -> Result<(), Error> {
    // Change this string to the Public Key value in your bot dashboard
    let verifier =
        Verifier::new("67c6bd767ca099e79efac9fcce4d2022a63bf7dea780e7f3d813f694c1597089");

    // Setup an HTTP server and listen for incoming interaction requests
    // Choose any port here (but be consistent with the interactions endpoint URL in your bot
    // dashboard)
    let server = tiny_http::Server::http("0.0.0.0:8787")?;
    let mut body = Vec::new();
    loop {
        let request = server.recv()?;
        if let Err(e) = handle_request(request, &mut body, &verifier) {
            eprintln!("Error while handling request: {e}");
        }
    }
}
