mod messages;
use axum::{
    http::header::HeaderMap,
    response::Json as response_json,
    Router,
    routing::get,
    routing::post,
};
use chrono::Utc;
use clap::Parser;
use hyper::StatusCode;
use messages::{
    ChatMessageSchema,
    GetApiResponse,
    GetChatMessagesResponse,
    RegionSchema,
    TimeFilterResponse
};
use rand::Rng;
use tracing::{event, Level};
use tracing_subscriber;
use uuid::Uuid;

pub const WS_UNCLASSIFIED_URL: &str = "wss://localhost:3030/root";
pub const DEFAULT_SERVE_IP: &str = "0.0.0.0";
pub const DEFAULT_SERVE_PORT: i32 = 80;

pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";
pub const TEST_DOMAIN_ID: &str = "chatsurferxmppunclass"; 

pub const GET_API_KEY_ROUTE: &str = "/api/auth/key";
pub const MESSAGES_ROUTE: &str = "/api/chat/messages/chatsurferxmppunclass/edge-view-test-room";
pub const NEW_MESSAGE_ROUTE: &str = "/api/chatserver/message";
pub const SEARCH_MESSAGES_ROUTE: &str = "/api/chat/messages/search";


fn build_region_array
(
    seed: i32,
    length: usize
) -> [messages::RegionSchema; messages::MAX_REGIONS] {
    let mut temp_vector: Vec<messages::RegionSchema> = Vec::new();
    let mut index: usize = 0;

    while index < length {
        temp_vector.insert(index, RegionSchema::new_test(seed as f32));
        index += 1;
    }

    temp_vector.try_into().unwrap_or_else(|temp_vector: Vec<RegionSchema>| panic!("Expected length of {} but it was {}", messages::MAX_REGIONS, temp_vector.len()))
}

fn build_geotag(seed: i32) -> messages::GeoTagSchema {
    messages::GeoTagSchema {
        anchorEnd:      seed as i64,
        anchorStart:    seed as i64,
        anchorText:     String::from(format!("Anchor text for GeoTag {}", seed)),
        confidence:     seed as f32,
        location:       messages::LocationSchema::init(
                            1.0,
                            messages::LocationType::Point),
        regions:        build_region_array(
                            seed,
                            messages::MAX_REGIONS),
        r#type: String::from(format!("PAL"))
    }
}

fn build_geotag_array(seed: i32) -> [messages::GeoTagSchema; messages::MAX_MESSAGE_GEOTAGS] {

    let new_array: [messages::GeoTagSchema; messages::MAX_MESSAGE_GEOTAGS] = [
        build_geotag(seed),
    ];

    new_array
}

fn build_chat_message
(
    seed: i32,
    new_name: &str,
    additional_text: &str,
) -> messages::ChatMessageSchema {

    let new_message: messages::ChatMessageSchema = messages::ChatMessageSchema {
        classification: String::from(UNCLASSIFIED_STRING),
        domainId:       String::from(TEST_DOMAIN_ID),
        geoTags:        build_geotag_array(seed),
        id:             Uuid::new_v4(),
        roomName:       String::from("Test room"),
        sender:         String::from(new_name),
        text:           String::from(format!("{}{}", 
            "This is some test message text.",
            additional_text)),
        threadId:       Uuid::new_v4(),
        timestamp:      Utc::now().to_string(),
        userId:         Uuid::new_v4()
    };

    new_message
} //end build_chat_message

fn build_get_messages_response() -> messages::GetChatMessagesResponse {
    let mut messages = Vec::new();

    messages.push(build_chat_message(
        25,
        "Austin",
        ""
    ));
    messages.push(build_chat_message(4, "Tyler", ""));
    messages.push(build_chat_message(7, "Joe", "test_keyword"));
    messages.push(build_chat_message(9, "Jeremy", ""));
    messages.push(build_chat_message(2, "Trevor", ""));
    messages.push(build_chat_message(4, "Justin", "test_keyword"));
    messages.push(build_chat_message(97856, "Ryan", ""));
    messages.push(build_chat_message(123, "Joseph", ""));
    messages.push(build_chat_message(432, "Rita", ""));
    messages.push(build_chat_message(654, "Matt", ""));


    messages::GetChatMessagesResponse {
        classification: messages::UNCLASSIFIED_STRING.to_string(),
        messages: messages
    }
}

fn search_messages(keywords: String) -> Vec<ChatMessageSchema> {
    let mut search_results: Vec<ChatMessageSchema> = Vec::new();

    let mut split_keywords: Vec<&str> = keywords.split(" ").collect();
    split_keywords.retain(|&x| x != "");
    event!(Level::DEBUG, "{:?}", split_keywords);

    let messages = build_get_messages_response().messages;

    for message in messages {
        if message.text.contains(split_keywords.first().unwrap()) {
            search_results.push(message);
        }
    }

    search_results
}

async fn handle_get_api_key() -> (StatusCode, String) {

    // Attempt to deserialize the request paylod.
    event!(Level::DEBUG, "Received Get API Key Request");

    let response: GetApiResponse = GetApiResponse {
        classification: String::from(UNCLASSIFIED_STRING),
        dn:             String::from("CN=Austin,O=Nine Hill Technology,ST=New York,C=US"),
        email:          String::from("austin.farrell@ninehilltech.com"),
        key:            String::from("a7B5siy9xY1dmN"),
        status:         messages::ApiKeyStatus::ACTIVE,
    };

    (StatusCode::OK, serde_json::to_string(&response).unwrap())
} // end handle_get_api_key

async fn handle_get_messages
(
    headers:    HeaderMap,
) -> (StatusCode, String) {
    event!(Level::DEBUG, "Received the Get Messages Request");

    if headers.contains_key("api-key") {
        let key_value = headers.get("api-key").unwrap();
        event!(Level::DEBUG, "{}", key_value.to_str().unwrap())
    }

    let response: messages::GetChatMessagesResponse;
    response = build_get_messages_response();

    event!(Level::DEBUG, "Sending the response");

    (StatusCode::OK, serde_json::to_string(&response).unwrap())
}

async fn handle_post_chat_message
(
    headers:    HeaderMap,
    payload:    String,
) -> (StatusCode, String) {

    if headers.contains_key("api-key") {
        let key_value = headers.get("api-key").unwrap();
        event!(Level::DEBUG, "{}", key_value.to_str().unwrap())
    }


    // Attempt to deserialize the request paylod.
    let request = messages::SendChatMessageRequest::from_string(payload.clone());
    event!(Level::DEBUG, "Received new message request from {}: {}", request.nickname, payload);

    
    //let num = rand::thread_rng().gen_range(0..2);
    let num = 0;
    
    match num {
        // 204 Successful case.
        0 => {
            event!(Level::DEBUG, "{}", serde_json::to_string("Hello").unwrap());
            (StatusCode::NO_CONTENT, serde_json::to_string("Hello").unwrap())
        },
        // 400 Bad Request case.
        1 => {
            let body = messages::ErrorCode400 {
                fieldErrors: vec![messages::FieldErrorSchema {
                    fieldName:          String::from("roomName"),
                    message:            String::from("Room name not found"),
                    messageArguments:   [String::from("I don't know what to put here")],
                    messageCode:        String::from("I don't know what to put here"),
                    rejectedValue:      String::from(request.roomName)
                }],
                ..Default::default()
            };

            event!(Level::DEBUG, "{}", serde_json::to_string(&body).unwrap());
            (StatusCode::BAD_REQUEST, serde_json::to_string(&body).unwrap())
        },
        // 429 Rate Exceeded case.
        _ => {
            event!(Level::DEBUG, "{}", serde_json::to_string("Hello 2").unwrap());
            (StatusCode::TOO_MANY_REQUESTS, serde_json::to_string("Hello 2").unwrap())
        },
    }
}

async fn handle_search_messages
(
    headers:    HeaderMap,
    payload:    String
) -> (StatusCode, String) {

    // Attempt to deserialize the request paylod.
    event!(Level::DEBUG, "Received Search Messages request: {}", payload);

    if headers.contains_key("api-key") {
        let key_value = headers.get("api-key").unwrap();
        event!(Level::DEBUG, "{}", key_value.to_str().unwrap())
    }

    let request = messages::SearchChatMessagesRequest::from_string(payload);
    
    //let num = rand::thread_rng().gen_range(0..2);
    let num = 0;
    
    match num {
        // 200 Successful case.
        0 => {
            let search_results = search_messages(request.keywordFilter.unwrap().query);
            let total: i32 = search_results.len() as i32;

            let body = messages::SearchChatMessagesResponse {
                classification:     String::from(UNCLASSIFIED_STRING),
                messages:           Some(search_results),
                nextCursorMark:     None,
                searchTimeFiler:    TimeFilterResponse {
                    endDateTime:    Utc::now().to_rfc3339()
                },
                total:              total,
            };


            event!(Level::DEBUG, "{}", serde_json::to_string(&body).unwrap());
            (StatusCode::OK, serde_json::to_string(&body).unwrap())
        },
        // 400 Bad Request case.
        1 => {
            let body = messages::ErrorCode400 {
                classification: String::from(UNCLASSIFIED_STRING),
                code:           400,

                fieldErrors:    vec![messages::FieldErrorSchema {
                    fieldName:          String::from("keywordFilter"),
                    message:            String::from("'*' or '?' not allowed as first character of a term"),
                    messageArguments:   [String::from("I don't know what to put here")],
                    messageCode:        String::from("ChatMessageSearchQueryStringIsInvalid"),
                    rejectedValue:      String::from("**")
                }],

                message:        String::from("The request contained 1 or more field validation errors."),
            };

            event!(Level::DEBUG, "{}", serde_json::to_string(&body).unwrap());
            (StatusCode::BAD_REQUEST, serde_json::to_string(&body).unwrap())
        },
        // 429 Rate Exceeded case.
        _ => {
            event!(Level::DEBUG, "{}", serde_json::to_string("Rate Exceeded").unwrap());
            (StatusCode::TOO_MANY_REQUESTS, serde_json::to_string("Rate Exceeded").unwrap())
        },
    }
} // end handle_search_messages

async fn handle_public_key_request() -> String {
    event!(Level::DEBUG, "Received the Get Public Key Request");
    String::from("{\"realm\":\"fmv\",\"public_key\":\"MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAzq/jsj5MTmOA9sW4YBJpv16yLPvznKLj3UqNXQ17WhukP5wu6GQyHMUSqNV8CAqGEA8TJpoQcpTCs8iaKxpfF1yORKdeuvCa/aJZpOw6TwsJZa1OWLONyJnOuPeZZNDUn+D7as+tS9ws7UP3AtROO8hkMS7+B3C90eXTWhZnkzEDSfDmfUxPMvYH/5yGUI4AtzbAGPMwiDOXOguXUSkV5TP7RXTZqrgHp3yvzBsbaWtjW9r4tfzXRHuGFXhlEgBdsBIzupaXrpfqIjHQXDhJ1NnI6KOQUTDi5t3VOhfZ8z6WXMPdqi/pvyzTenAshvoTR2rEti6KyLqwTdW6y1KFVQIDAQAB\",\"token-service\":\"https://app.fmvedgeview.net/keycloak/auth/realms/fmv/protocol/openid-connect\",\"account-service\":\"https://app.fmvedgeview.net/keycloak/auth/realms\",\"tokens-not-before\":0}")
} // end handle_public_key_request

/*
 * This struct describes the possible arguments accepted by the
 * WebSocket-TestServer service.
 */
#[derive(serde::Serialize)]
#[derive(Parser, Debug)]
struct Args {
    // This field indicates the IP address from which to serve
    // client requests.
    #[arg(long = "client_serve_ip", default_value_t = String::from(DEFAULT_SERVE_IP))]
    client_serve_ip:    String,
    
    // This field sets the port number from which to serve requests
    // from a client.
    #[arg(long = "client_port", default_value_t = DEFAULT_SERVE_PORT)]
    client_port:        i32,
}

impl Args {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[tokio::main]
//-> Result<(), Box<dyn std::error::Error + Send + Sync>>
async fn main()  {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Parse the command line arguments and log them.
    let args = Args::parse();
    event!(Level::DEBUG, "{}", args.to_json());

    // Construct the address string we're going to serve from.
    let serve_address: String = format!("{}:{}", args.client_serve_ip, args.client_port);
    event!(Level::DEBUG, "Hosting at {}", serve_address);


    let test_route = Router::new()
        .route(GET_API_KEY_ROUTE, get(handle_get_api_key))
        .route(MESSAGES_ROUTE, get(handle_get_messages))
        .route(NEW_MESSAGE_ROUTE, post(handle_post_chat_message))
        .route(SEARCH_MESSAGES_ROUTE, post(handle_search_messages))
        .route("/auth/realms/fmv", get(handle_public_key_request));

    
    let axum_listener = tokio::net::TcpListener::bind(serve_address).await.unwrap();

    match axum::serve(axum_listener, test_route).await {
        Ok(()) => {
            event!(Level::DEBUG, "Serving requests...");
        }
        Err(e) => {
            event!(Level::ERROR, "Error in the Axum server: {}" , e);
        }
    }





    // // Construct the address to host HTTP requests from.
    // let serve_socket = SocketAddr::from_str(serve_address.as_str()).unwrap();

    // // Set up the HTTP listener.
    // let listener = TcpListener::bind(serve_socket).await?;

    // // We start a loop to continuously accept incoming connections
    // event!(Level::DEBUG, "Serving requests at {}", serve_address);
    // loop {
    //     let (stream, _) = listener.accept().await?;

    //     // Use an adapter to access something implementing `tokio::io` traits as if they implement
    //     // `hyper::rt` IO traits.
    //     let io = TokioIo::new(stream);

    //     // Spawn a tokio task to serve multiple connections concurrently
    //     tokio::task::spawn(async move {
    //         // Finally, we bind the incoming connection to our `hello` service
    //         if let Err(err) = http1::Builder::new()
    //             // `service_fn` converts our function in a `Service`
    //             .serve_connection(io, service_fn(request_handler))
    //             .await
    //         {
    //             eprintln!("Error serving connection: {:?}", err);
    //         }
    //     });
    // }
}