// General and standard crates
use chrono::{ Date, DateTime, Utc };
use clap::Parser;
use futures::{SinkExt, StreamExt};
use std::str::FromStr;
use std::{any, convert::Infallible, io, net::SocketAddr};
use std:: {
    io::{ prelude::*, BufReader },
    net::TcpStream,
    //net::TcpListener,
};
use uuid::Uuid;

// Logging crates
use tracing::{event, span, Level};
use tracing_subscriber;

// HTTP crates
use axum::{
    extract::Json as extract_json,
    handler::Handler,
    response::Json as response_json,
    Router,
    routing::get,
    routing::post,
};
use http_body_util::{
    combinators::BoxBody,
    BodyExt,
    Empty,
    Full
};
use hyper::{
    body::Bytes,
    body::Frame,
    Method,
    Request,
    Response,
    server::conn::http1,
    service::service_fn,
    StatusCode,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// Project-specific crates
mod messages;

use messages::{
    ChatMessageSchema, GeoTagSchema, GetChatMessagesResponse, LocationCoordinatesSchema, LocationSchema, LocationType, RegionSchema, SearchChatMessagesRequest,
    SendChatMessageRequest,
};

pub const WS_UNCLASSIFIED_URL: &str = "wss://localhost:3030/root";
pub const DEFAULT_SERVE_IP: &str = "0.0.0.0";
pub const DEFAULT_SERVE_PORT: i32 = 80;

pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";
pub const TEST_DOMAIN_ID: &str = "chatsurferxmppunclass"; 

pub const MESSAGES_ROUTE: &str = "/api/chat/messages/somedomain/Test_Room";
pub const NEW_MESSAGE_ROUTE: &str = "/api/chatserver/message";


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

    messages.push(build_chat_message(25, "Austin", ""));
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


fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}





async fn request_handler(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    
    let response: messages::GetChatMessagesResponse;

    
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/api/chat/messages/somedomain/Test_Room") => {
            event!(Level::DEBUG, "Caught the GET Request");

            // Construct a test chat message object to send back to the client.
            response = build_get_messages_response();

            let message_json = serde_json::to_string(&response).unwrap();

            let boxed = Full::new(message_json.into())
                .map_err(|never| match never {})
                .boxed();
            
            Ok(Response::new(boxed))
        }
        (&Method::POST, "/api/chat/messages/search") => {
            event!(Level::DEBUG, "Caught the POST Request");

            //let whole_body = req.body().collect().await?.to_bytes();

            //let (head, body, _tail) = unsafe { whole_body.align_to::<SearchChatMessagesRequest>() };

            //let search_request: SearchChatMessagesRequest = body[0];
            //event!(Level::DEBUG, "Search request: {}", search_request);



            //Construct test chat message objects to send back to the client.
            response = build_get_messages_response();

            let trimmed_response: messages::GetChatMessagesResponse = GetChatMessagesResponse::new();

            for message in response.messages {
                //if message.text.contains(req.k)
            }

            
            let boxed = Full::new(String::from("Unimplemented").into())
            .map_err(|never| match never {})
            .boxed();
        
        Ok(Response::new(boxed))
        }
        _ => {
            event!(Level::DEBUG, "NOT FOUND");
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;

            Ok(not_found)
        }
    }
}

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

async fn handle_users() -> response_json<GetChatMessagesResponse> {
    event!(Level::DEBUG, "Received the Get Messages Request");
    let response: messages::GetChatMessagesResponse;
    response = build_get_messages_response();

    event!(Level::DEBUG, "Sending the response");
    response_json(response)
}

async fn handle_post_chat_message(extract_json(payload): extract_json<SendChatMessageRequest>) {

    // Attempt to deserialize the request paylod.
    event!(Level::DEBUG, "Received new message request: {}", payload);
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


    let test_route = Router::new()
        .route(MESSAGES_ROUTE, get(handle_users))
        .route(NEW_MESSAGE_ROUTE, post(handle_post_chat_message));


    let axum_listener = tokio::net::TcpListener::bind(serve_address).await.unwrap();

    axum::serve(axum_listener, test_route).await.unwrap();





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