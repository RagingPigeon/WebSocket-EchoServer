mod messages;
use axum::{
    extract::ws::{
        Message,
        WebSocketUpgrade,
        WebSocket,
    },
    http::header::HeaderMap,
    response::Json as response_json,
    response::Response,
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
use std::{
    thread,
    time::{
        Duration,
        self,
    }
};
use thread_id;
use tracing::{event, Level};
use tracing_subscriber;
use uuid::Uuid;

pub const WS_UNCLASSIFIED_URL: &str = "wss://localhost/root";
pub const DEFAULT_SERVE_IP: &str = "0.0.0.0";
pub const DEFAULT_SERVE_PORT: i32 = 443;

pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";
pub const TEST_ROOM_NAME: &str = "edge-view-test-room";
pub const TEST_DOMAIN_ID: &str = "chatsurferxmppunclass"; 
pub const TEST_KEYWORD: &str = "Antediluvian";

pub const GET_API_KEY_ROUTE: &str = "/api/auth/key";
pub const MESSAGES_ROUTE: &str = "/api/chat/messages/chatsurferxmppunclass/edge-view-test-room";
pub const NEW_MESSAGE_ROUTE: &str = "/api/chatserver/message";
pub const SEARCH_MESSAGES_ROUTE: &str = "/api/chatsearch/messages/search";

pub const WS_SINGLE_ROOM_ROUTE: &str = "/topic/chat-messages-room/chatsurferxmppunclass/edge-view-test-room";

pub const SECONDS_BETWEEN_WEBSOCKET_UPDATE: u64 = 1;

pub const MAX_REGIONS: usize = 5;

fn build_region_array(
    seed:   i32,
    length: usize
) -> Vec<messages::RegionSchema> {
    let mut temp_vector: Vec<messages::RegionSchema> = Vec::new();
    let mut index: usize = 0;

    while index < length {
        temp_vector.insert(index, RegionSchema::new_test(seed as f32));
        index += 1;
    }

    temp_vector
}

fn build_geotag(seed: i32) -> messages::GeoTagSchema {
    messages::GeoTagSchema {
        anchor_end:      seed as i64,
        anchor_start:    seed as i64,
        anchor_text:     String::from(format!("Anchor text for GeoTag {}", seed)),
        confidence:     seed as f32,
        location:       messages::LocationSchema::init(
                            1.0,
                            messages::LocationType::Point),
        regions:        build_region_array(
                            seed,
                            MAX_REGIONS),
        r#type: String::from(format!("PAL"))
    }
}

fn build_geotag_array(seed: i32) -> Vec<messages::GeoTagSchema> {
    vec!(build_geotag(seed))
}

fn build_chat_message(
    seed: i32,
    new_name: &str,
    additional_text: &str,
) -> messages::ChatMessageSchema {

    messages::ChatMessageSchema {
        classification: String::from(UNCLASSIFIED_STRING),
        domain_id:      String::from(TEST_DOMAIN_ID),
        geo_tags:       Some(build_geotag_array(seed)),
        id:             Uuid::new_v4().to_string(),
        room_name:      String::from(TEST_ROOM_NAME),
        sender:         String::from(new_name),
        text:           String::from(format!("{}{}", 
            "This is some test message text.",
            additional_text)),
        thread_id:      Some(Uuid::new_v4().to_string()),
        timestamp:      Utc::now().to_string(),
        user_id:        Uuid::new_v4().to_string(),
        private:        false,
    }
} //end build_chat_message

fn build_get_messages_response() -> messages::GetChatMessagesResponse {
    let mut messages = Vec::new();

    messages.push(build_chat_message(
        25,
        "Austin",
        TEST_KEYWORD
    ));
    messages.push(build_chat_message(4, "Tyler", ""));
    messages.push(build_chat_message(7, "Joe", TEST_KEYWORD));
    messages.push(build_chat_message(9, "Jeremy", ""));
    messages.push(build_chat_message(2, "Trevor", ""));
    messages.push(build_chat_message(4, "Justin", TEST_KEYWORD));
    messages.push(build_chat_message(97856, "Ryan", ""));
    messages.push(build_chat_message(123, "Joseph", ""));
    messages.push(build_chat_message(432, "Rita", ""));
    messages.push(build_chat_message(654, "Matt", ""));


    messages::GetChatMessagesResponse {
        classification: messages::UNCLASSIFIED_STRING.to_string(),
        messages: messages,
        domain_id: String::from(TEST_DOMAIN_ID),
        room_name: String::from(TEST_ROOM_NAME),
        private: false,
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
        status:         serde_json::to_string(&messages::ApiKeyStatus::Active).unwrap(),
    };

    (StatusCode::OK, serde_json::to_string(&response).unwrap())
} // end handle_get_api_key

async fn handle_get_messages(
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

async fn handle_post_chat_message(
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
                // field_errors: vec![messages::FieldErrorSchema {
                //     field_name:          String::from("roomName"),
                //     message:            String::from("Room name not found"),
                //     message_arguments:   vec!(String::from("I don't know what to put here")),
                //     message_code:        String::from("I don't know what to put here"),
                //     rejected_value:      String::from(request.room_name)
                // }],
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

async fn handle_search_messages(
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
            let search_results = search_messages(request.keyword_filter.unwrap().query);
            let total: i32 = search_results.len() as i32;

            let body = messages::SearchChatMessagesResponse {
                classification:     String::from(UNCLASSIFIED_STRING),
                messages:           Some(search_results),
                next_cursor_mark:   None,
                search_time_filter: TimeFilterResponse {
                    end_date_time:  Utc::now().to_rfc3339()
                },
                total:              total,
            };


            event!(Level::DEBUG, "{}", serde_json::to_string(&body).unwrap());
            (StatusCode::OK, serde_json::to_string(&body).unwrap())
        },
        // 400 Bad Request case.
        1 => {
            // let body = messages::ErrorCode400 {
            //     classification: String::from(UNCLASSIFIED_STRING),
            //     code:           400,

            //     field_errors:    vec![messages::FieldErrorSchema {
            //         field_name:          String::from("keywordFilter"),
            //         message:            String::from("'*' or '?' not allowed as first character of a term"),
            //         message_arguments:   vec!(String::from("I don't know what to put here")),
            //         message_code:        String::from("ChatMessageSearchQueryStringIsInvalid"),
            //         rejected_value:      String::from("**")
            //     }],

            //     message:        String::from("The request contained 1 or more field validation errors."),
            // };

            // let body = messages::ErrorCode400 {
            //     classification: String::from("SECRET//NOFORN"),
            //     code:           400,

            //     // field_errors:    vec![messages::FieldErrorSchema {
            //     //     field_name:          String::from("keywordFilter"),
            //     //     message:            String::from("'*' or '?' not allowed as first character of a term"),
            //     //     message_arguments:   vec!(String::from("I don't know what to put here")),
            //     //     message_code:        String::from("ChatMessageSearchQueryStringIsInvalid"),
            //     //     rejected_value:      String::from("**")
            //     // }],

            //     message:        String::from("The request contained 1 or more field validation errors."),
            // };
            // let body = String::from("{\"classification\":\"SECRET//NOFORN\",\"code\":400,\"message\":\"Request body is missing or not readable.\"}");
            let body = String::from("{\"classification\":\"SECRET//NOFORN\",\"code\":400,\"message\":\"Request body is missing or not readable.\"}");

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

async fn serve_ws_single_room(
    mut socket: axum::extract::ws::WebSocket
) {
    loop {
        // We will periodically send messages to the client to simulate events
        // taking place within a ChatSurfer chat room.
        thread::sleep(Duration::from_secs(SECONDS_BETWEEN_WEBSOCKET_UPDATE));

        // Send a randomly generated chat message to the client.

        let random_seed = rand::random::<i32>();

        let message = build_chat_message(
            random_seed.clone(),
            "Austin",
            random_seed.clone().to_string().as_str()
        );

        match socket.send(Message::Text(
            message.try_to_json().unwrap()
        )).await {
            Ok(()) => {
                event!(Level::DEBUG, "Successfully sent message {} to client.", random_seed);
            }
            Err(e) => {
                event!(Level::ERROR, "Error - could not send the response to the client: {}", e);
            }
        }
    }
} // end serve_ws_single_room

async fn serve_ws_single_room_upgrade_handler(
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| serve_ws_single_room(socket))
} // end serve_ws_single_room_upgrade_handler

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

async fn test() {

    loop {
        
        event!(Level::DEBUG, "Thread {}: spinning", thread_id::get());
        
        thread::sleep(time::Duration::from_secs(10));
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
        .route("/auth/realms/fmv", get(handle_public_key_request))
        .route(GET_API_KEY_ROUTE, get(handle_get_api_key))
        .route(MESSAGES_ROUTE, get(handle_get_messages))
        .route(NEW_MESSAGE_ROUTE, post(handle_post_chat_message))
        .route(SEARCH_MESSAGES_ROUTE, post(handle_search_messages))
        .route(WS_SINGLE_ROOM_ROUTE, get(serve_ws_single_room_upgrade_handler))
        .route("/connect", get(serve_ws_single_room_upgrade_handler))
        .route("/test", get(test));

    
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