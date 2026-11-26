use oxcord::async_trait;
use oxcord::builder::*;
use oxcord::interactions_endpoint::{
    process_interaction,
    InteractionHandler,
    InteractionProcessingError,
    Verifier,
};
use oxcord::model::application::*;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Modern interaction handler for Discord HTTP webhooks.
struct BotHandler;

#[async_trait]
impl InteractionHandler for BotHandler {
    /// Responds to Slash Commands.
    async fn on_command(&self, command: &CommandInteraction) -> Option<CreateInteractionResponse> {
        println!("Received command: {}", command.data.name);

        Some(CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(
            format!(
                "Hello from Oxcord Interactions Webhook Server! Invoked by <@{}>",
                command.user.id
            ),
        )))
    }

    /// Responds to dynamic Autocomplete choices.
    async fn on_autocomplete(
        &self,
        autocomplete: &CommandInteraction,
    ) -> Option<CreateInteractionResponse> {
        println!("Received autocomplete for: {}", autocomplete.data.name);

        Some(CreateInteractionResponse::Autocomplete(
            CreateAutocompleteResponse::default().set_choices(vec![
                "Option A".into(),
                "Option B".into(),
                "Option C".into(),
            ]),
        ))
    }

    /// Responds to Modal Submissions.
    async fn on_modal(&self, modal: &ModalInteraction) -> Option<CreateInteractionResponse> {
        println!("Received modal submission with custom_id: {}", modal.data.custom_id);

        Some(CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Modal submission received and processed!"),
        ))
    }
}

async fn handle_request(
    mut request: tiny_http::Request,
    body: &mut Vec<u8>,
    verifier: &Verifier,
    handler: &BotHandler,
) -> Result<(), Error> {
    println!("Received request from {:?}", request.remote_addr());

    // Read raw body bytes
    body.clear();
    request.as_reader().read_to_end(body)?;

    // Extract headers for cryptographic verification
    let find_header =
        |name| Some(request.headers().iter().find(|h| h.field.equiv(name))?.value.as_str());
    let signature = match find_header("X-Signature-Ed25519") {
        Some(s) => s,
        None => {
            request.respond(tiny_http::Response::empty(401))?;
            return Ok(());
        },
    };
    let timestamp = match find_header("X-Signature-Timestamp") {
        Some(t) => t,
        None => {
            request.respond(tiny_http::Response::empty(401))?;
            return Ok(());
        },
    };

    // Process interaction using zero-alloc verification and the modern handler interface
    let output = match process_interaction(verifier, handler, signature, timestamp, body).await {
        Ok(out) => out,
        Err(InteractionProcessingError::Verification(e)) => {
            eprintln!("Signature verification failed: {e}");
            request.respond(tiny_http::Response::empty(401))?;
            return Ok(());
        },
        Err(InteractionProcessingError::InvalidJson(e)) => {
            eprintln!("Invalid interaction JSON: {e}");
            request.respond(tiny_http::Response::empty(400))?;
            return Ok(());
        },
        Err(InteractionProcessingError::Serialization(e)) => {
            eprintln!("Failed to serialize interaction response: {e}");
            request.respond(tiny_http::Response::empty(500))?;
            return Ok(());
        },
    };

    // Return the response back to Discord over HTTP
    let header =
        format!("Content-Type: {}", output.content_type).parse::<tiny_http::Header>().unwrap();
    let resp = tiny_http::Response::from_data(output.body)
        .with_status_code(output.status)
        .with_header(header);

    request.respond(resp)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Public key from the Discord Developer Portal dashboard
    let verifier =
        Verifier::new("67c6bd767ca099e79efac9fcce4d2022a63bf7dea780e7f3d813f694c1597089");
    let handler = BotHandler;

    println!("Starting Oxcord Interactions Webhook Server on 0.0.0.0:8787...");
    let server = tiny_http::Server::http("0.0.0.0:8787")?;
    let mut body = Vec::new();

    loop {
        let request = server.recv()?;
        if let Err(e) = handle_request(request, &mut body, &verifier, &handler).await {
            eprintln!("Error while handling request: {e}");
        }
    }
}
