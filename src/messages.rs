use anyhow::{
    Context,
    Result,
};

use http::StatusCode;
use serde::{ Deserialize, Serialize };
use std::{
    collections::HashMap,
    fmt
};
use strum_macros::{ EnumString, Display };
use tracing::{ event, Level };
use uuid::Uuid;

/// The ChatSurfer API limits client requests to a certain number
/// every minute.
/// 
/// <https://chatsurfer.nro.mil/apidocs#section/(U)-Rate-Limiting>
pub const MAX_REQUESTS_PER_MINUTE: i32 = 60;

// Classification strings
pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";

// #############################################################################
// #############################################################################
//                              Error Messages
// #############################################################################
// #############################################################################

//==============================================================================
// ErrorCode400
//==============================================================================

/// This structure represents an HTTP 400 Bad Request message received
/// from ChatSurfer.
#[derive(Serialize, Deserialize)]
pub struct ErrorCode400 {
    pub classification: String,
    pub code:           u16,
    
    #[serde(rename = "fieldErrors")]
    pub field_errors:   Vec<FieldErrorSchema>,
    pub message:        String,
}

impl Default for ErrorCode400 {
    fn default() -> Self {
        ErrorCode400 {
            classification: String::from(UNCLASSIFIED_STRING),
            code:           400,
            field_errors:   Vec::new(),
            message:        String::from("Bad Request"),
        }
    }
}

/// Implement the trait fmt::Display for the struct ErrorCode400
/// so that these structs can be easily printed to consoles.
impl fmt::Display for ErrorCode400 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl ErrorCode400 {
    pub fn test(source: String) -> ErrorCode400 {
        ErrorCode400 {
            classification: String::from(UNCLASSIFIED_STRING),
            code:           400,
            field_errors:   vec!(FieldErrorSchema::from_string(source.clone())),
            message:        source.clone(),
        }
    }

    /// This method constructs a JSON string from the
    /// ErrorCode400's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the ErrorCode400 struct, and return
        // any error with a context message.
        Ok(serde_json::to_string(self)
            .context("Unable to convert the ErrorCode400 struct to a string.")?)
    }
    
    /// This method attempts to construct a ErrorCode400
    /// structure from the given String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_string(source: String) -> Result<ErrorCode400, anyhow::Error> {
        Ok(serde_json::from_str::<ErrorCode400>(&source)
            .with_context(|| format!("Unable to create ErrorCode400 struct from String {}", source))?)
    } // end try_from_string
} // end ErrorCode400

//==============================================================================
// ErrorCode404
//==============================================================================

/// This structure represents an HTTP 404 Not Found message received
/// from ChatSurfer.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorCode404 {
    pub classification: String,
    pub code:           u16,
    pub message:        String
}

impl std::fmt::Display for ErrorCode404 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl std::error::Error for ErrorCode404 {}

impl ErrorCode404 {
    /// This method attempts to construct a ErrorCode404
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_string(source: String) -> Result<ErrorCode404, anyhow::Error> {
        Ok(serde_json::from_str::<ErrorCode404>(&source)
            .with_context(|| format!("Unable to create ErrorCode404 struct from String {}", source))?)
    }
    
    /// This method constructs a JSON string from the
    /// ErrorCode404's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        // Attempt to serialize the ErrorCode404 struct, and return
        // any error with a context message.
        Ok(serde_json::to_string(self)
            .context("Unable to convert the ErrorCode404 struct to a string.")?)
    }
}

// #############################################################################
// #############################################################################
//                              API Key Messages
// #############################################################################
// #############################################################################

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum ApiKeyStatus {
    #[strum(serialize = "ACTIVE")]
    ACTIVE,
    #[strum(serialize = "DISABLED")]
    DISABLED,
    #[strum(serialize = "PENDING")]
    PENDING,
}

#[derive(Serialize, Deserialize)]
pub struct GetApiResponse {
    pub classification: String,
    
    // The Distinguished Name of the certificate used to
    // create the API key.
    pub dn:             String,
    pub email:          String,
    pub key:            String,

    // The status of the API Key.
    pub status:         String,
}

/// Implement the trait fmt::Display for the struct GetApiResponse
/// so that these structs can be easily printed to consoles.
impl fmt::Display for GetApiResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetApiResponse {
    /// This method attempts to construct a GetApiResponse
    /// structure from the given JSON String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_json(json: String) -> Result<GetApiResponse, anyhow::Error> {
        Ok(serde_json::from_str::<GetApiResponse>(&json)
            .with_context(|| format!("Unable to create GetApiResponse struct from String {}", json))?)
    }
    
    /// This method constructs a JSON string from the
    /// GetApiResponse's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the GetApiResponse struct to a string.")?)
    }
} // end GetApiResponse

// =============================================================================
// struct SendChatMessageRequest
// =============================================================================

/// The SendChatMessageRequest structure represents an HTTP request that can
/// be sent to ChatSurfer to create a new chat message within the defined
/// chat room.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Send%20Chat%20Message>
#[derive(Serialize, Deserialize)]
pub struct SendChatMessageRequest {
    pub classification: String,

    #[serde(rename = "domainId")]
    pub domain_id:      String,
    pub message:        String,
    pub nickname:       String,

    #[serde(rename = "roomName")]
    pub room_name:      String
}

/// Implement the trait Default for the struct SendChatMessageRequest
/// so that we can fall back on default values.
impl Default for SendChatMessageRequest {
    fn default() -> SendChatMessageRequest {
        SendChatMessageRequest {
            classification: String::from(UNCLASSIFIED_STRING),
            domain_id:      String::new(),
            message:        String::new(),
            nickname:       String::from("Edge View"),
            room_name:      String::new()
        }
    }
}

/// Implement the trait fmt::Display for the struct SendChatMessageRequest
/// so that these structs can be easily printed to consoles.
impl fmt::Display for SendChatMessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SendChatMessageRequest {
    pub fn from_string(json: String) -> SendChatMessageRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }

    /// This method constructs a JSON string from the
    /// SendChatMessageRequest's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the SendChatMessageRequest struct to a string.")?)
    }
} //end SendChatMessageRequest

// =============================================================================
// GetChatMessagesResponse
// =============================================================================

/// The GetChatMessagesResponse structure defines the response we
/// expect to receive from a successful Get Chat messages By Room request.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20Chat%20Messages%20By%20Room>
#[derive(Serialize, Deserialize)]
pub struct GetChatMessagesResponse {
    pub classification: String,
    pub messages:       Vec<ChatMessageSchema>,

    #[serde(rename = "domainId")]
    pub domain_id:      String,
    pub private:        bool,
    
    #[serde(rename = "roomName")]
    pub room_name:      String,
}

impl fmt::Display for GetChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GetChatMessagesResponse {
    pub fn test(source: String) -> GetChatMessagesResponse {
        GetChatMessagesResponse {
            classification: String::from("UNCLASSIFIED"),
            messages:       vec!(),
            domain_id:      source.clone(),
            private:        false,
            room_name:      source,
        }
    }

    pub fn try_from_string(source: String) -> Result<GetChatMessagesResponse, anyhow::Error> {
        Ok(serde_json::from_str::<GetChatMessagesResponse>(&source)
            .with_context(|| format!("Unable to create GetChatMessagesResponse struct from String {}", source))?)
    }

    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the GetChatMessagesResponse struct to a string.")?)
    }
} // end GetChatMessagesResponse

// =============================================================================
// SearchChatMessagesRequest
// =============================================================================

/// The SearchChatMessagesRequest structure represents an HTTP request that can
/// be sent to ChatSurfer to search for chat messages based on the given
/// search criteria.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesRequest {
    pub cursor:             Option<String>,
    
    #[serde(rename = "filesOnly")]
    pub files_only:         Option<bool>,
    
    #[serde(rename = "highlightResults")]
    pub highlight_results:  Option<bool>,
    
    #[serde(rename = "keywordFilter")]
    pub keyword_filter:     Option<KeywordFilter>,
    pub limit:              Option<i32>,
    pub location:           Option<LocationSchema>,
    
    #[serde(rename = "locationFilter")]
    pub location_filter:    Option<bool>,
    
    #[serde(rename = "mentionFilter")]
    pub mention_filter:     Option<MentionFilter>,
    
    #[serde(rename = "requestGeoTags")]
    pub request_geo_tags:   Option<bool>,
    
    #[serde(rename = "roomFilter")]
    pub room_filter:        Option<DomainFilterDetail>,
    
    #[serde(rename = "senderFilter")]
    pub sender_filter:      Option<DomainFilterDetail>,
    pub sort:               Option<SortFilter>,
    
    #[serde(rename = "threadIdFilter")]
    pub thread_id_filter:   Option<ThreadIdFilter>,
    
    #[serde(rename = "timeFilter")]
    pub time_filter:        Option<TimeFilterRequest>,
    
    #[serde(rename = "userIdFilter")]
    pub user_id_filter:     Option<UserIdFilter>,

    #[serde(rename = "UserHighClassification")]
    pub user_high_classification:   String,
}

impl Default for SearchChatMessagesRequest {
    fn default() -> Self {
        SearchChatMessagesRequest {
            cursor:             None,
            files_only:         None,
            highlight_results:  None,
            keyword_filter:     None,
            limit:              None,
            location:           None,
            location_filter:    None,
            mention_filter:     None,
            request_geo_tags:   None,
            room_filter:        None,
            sender_filter:      None,
            sort:               None,
            thread_id_filter:   None,
            time_filter:        None,
            user_id_filter:     None,
            user_high_classification:   String::from("Test"),
        }
    }
}

/// Implement the trait fmt::Display for the struct SearchChatMessagesRequest
/// so that these structs can be easily printed to consoles.
impl fmt::Display for SearchChatMessagesRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SearchChatMessagesRequest {
    pub fn from_string(json: String) -> SearchChatMessagesRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }

    /// This method constructs a JSON string from the SearchChatMessagesRequest's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the SearchChatMessagesRequest struct to a string.")?)
    }
}

// =============================================================================
// SearchChatMessagesResponse
// =============================================================================

/// The SearchChatMessagesResponse structure represents the response we
/// expect to receive from ChatSurfer upon a successful Search Chat messages
/// request.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesResponse {
    pub classification:     String,
    pub messages:           Option<Vec<ChatMessageSchema>>,

    #[serde(rename = "nextCursorMark")]
    pub next_cursor_mark:   Option<String>,

    #[serde(rename = "searchTimeFiler")]
    pub search_time_filter: TimeFilterResponse,
    pub total:              i32,
}

/// Implement the trait fmt::Display for the struct SearchChatMessagesResponse
/// so that these structs can be easily printed to consoles.
impl fmt::Display for SearchChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl SearchChatMessagesResponse {
    /// This method constructs a JSON string from the
    /// SearchChatMessagesResponse's fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the SearchChatMessagesResponse struct to a string.")?)
    }

    /// This method attempts to construct a SearchChatMessagesResponse
    /// structure from the given String parameter.
    /// 
    /// If a failure occurs, the None variant will be returned.
    pub fn try_from_string(source: String)
        -> Result<SearchChatMessagesResponse, anyhow::Error> {
        Ok(serde_json::from_str::<SearchChatMessagesResponse>(&source)
            .with_context(||
                format!("Unable to create SearchChatMessagesRequest struct from String {}", source))?)
    } // end try_from_string
} // end SearchChatMessagesResponse

/// This enumeration defines the types of responses we can receive
/// from ChatSurfer.
/// 
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20API%20Key>
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Send%20Chat%20Message>
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20Chat%20Messages%20By%20Room>
/// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
pub enum ChatSurferResponseType {
    GetApiKey           { body: GetApiResponse },
    SendChatMessage,
    GetChatMessages     { body: GetChatMessagesResponse },
    SearchChatMessages  { body: SearchChatMessagesResponse },
    Failure400          { body: ErrorCode400 },
    Failure404          { body: ErrorCode404 },
    Failure429,
}

// #############################################################################
// #############################################################################
//                           Supporting Structures
// #############################################################################
// #############################################################################
//==============================================================================
// ChatMessageSchema
//==============================================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessageSchema {
    pub classification: String,
    
    #[serde(rename = "domainId")]
    pub domain_id:      String,
    
    #[serde(rename = "geoTags")]
    pub geo_tags:       Option<Vec<GeoTagSchema>>,
    pub id:             String,
    
    #[serde(rename = "roomName")]
    pub room_name:      String,
    pub sender:         String,
    pub text:           String,
    
    #[serde(rename = "threadId")]
    pub thread_id:      Option<String>,
    pub timestamp:      String,
    
    #[serde(rename = "userId")]
    pub user_id:        String,
    pub private:        bool,
}

impl fmt::Display for ChatMessageSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl ChatMessageSchema {
    pub fn test(source: String, seed: f32) -> ChatMessageSchema {
        ChatMessageSchema {
            classification: String::from("UNCLASSIFIED"),
            domain_id:      String::from(source.clone()),
            geo_tags:       Some(vec!(GeoTagSchema::test(source.clone(), seed))),
            id:             String::from(source.clone()),
            room_name:      String::from(source.clone()),
            sender:         String::from(source.clone()),
            text:           String::from(source.clone()),
            thread_id:      Some(String::from(source.clone())),
            timestamp:      String::from(source.clone()),
            user_id:        String::from(source.clone()),
            private:        false,
        }
    }
    
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the ChatMessageSchema struct to a string.")?)
    }

    pub fn try_from_json(json: String)
        -> Result<ChatMessageSchema, anyhow::Error> {
        Ok(serde_json::from_str::<ChatMessageSchema>(&json)
            .with_context(|| format!(
                "Unable to create GetUsersRequest struct from String {}",
                json))?
        )
    }
} // end ChatMessageSchema

//==============================================================================
// FieldErrorSchema
//==============================================================================
#[derive(Serialize, Deserialize)]
pub struct FieldErrorSchema {
    #[serde(rename = "fieldName")]
    pub field_name:         String,
    pub message:            String,
    
    #[serde(rename = "messageArguments")]
    pub message_arguments:  Vec<String>,
    
    #[serde(rename = "messageCode")]
    pub message_code:       String,
    
    #[serde(rename = "rejectedValue")]
    pub rejected_value:     String
}

impl Default for FieldErrorSchema {
    fn default() -> Self {
        FieldErrorSchema {
            field_name:         String::new(),
            message:            String::new(),
            message_arguments:  Vec::new(),
            message_code:       String::new(),
            rejected_value:     String::new(),
        }
    }
}

impl FieldErrorSchema {
    pub fn from_string(source: String) -> FieldErrorSchema {
        FieldErrorSchema {
            field_name:         source.clone(),
            message:            source.clone(),
            message_arguments:  vec!(source.clone()),
            message_code:       source.clone(),
            rejected_value:     source.clone(),
        }
    }
}

//==============================================================================
// NetworkId
//==============================================================================
/// This enum lists the possible values for a Domain's network ID.
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum NetworkId {
    #[strum(serialize = "bices")]
    Bices,

    #[strum(serialize = "cxk")]
    Cxk,

    #[strum(serialize = "sipr")]
    Sipr,

    #[strum(serialize = "jwics")]
    Jwics,

    #[strum(serialize = "unclass")]
    Unclass,
}

//==============================================================================
// JoinStatus
//==============================================================================
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum JoinStatus {
    #[strum(serialize = "JOINED")]
    Joined,

    #[strum(serialize = "NOT_JOINED")]
    NotJoined,
}

//==============================================================================
// LocationCoordinatesSchema
//==============================================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct LocationCoordinatesSchema {
    #[serde(rename = "type")]
    r#type:                 LocationType,

    // The first entry represents the coordinates for a single point.
    point_coordinates:      Vec<f32>,
    
    // The second entry represents a set of points for a polygon.
    polygon_coordinates:    Vec<Vec<f32>>,

}

impl fmt::Display for LocationCoordinatesSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl LocationCoordinatesSchema {

    pub fn init(seed: f32, r#type: &LocationType) -> LocationCoordinatesSchema {
        match r#type {
            LocationType::Point => {
                LocationCoordinatesSchema {
                    r#type: LocationType::Point,

                    point_coordinates: LocationCoordinatesSchema::new_point(seed),

                    // Zeroize the alternate coordinate structure.
                    polygon_coordinates: vec!(vec!(0.0))
                }
            }
            LocationType::Polygon => {
                LocationCoordinatesSchema {
                    r#type: LocationType::Polygon,

                    polygon_coordinates: LocationCoordinatesSchema::new_polygon(seed),
                    
                    // Zeroize the alternate coordinate structure.
                    point_coordinates: vec!(0.0)
                }
            }
        }
    } //end init

    pub fn new_point(seed: f32) -> Vec<f32> {
        vec!(seed)
    }

    pub fn new_polygon(seed: f32) -> Vec<Vec<f32>> {
        vec!(vec!(seed))
    }

    pub fn test(seed: f32) -> LocationCoordinatesSchema {
        LocationCoordinatesSchema {
            r#type:                 LocationType::Point,
            point_coordinates:      vec!(seed.clone()),
            polygon_coordinates:    vec!(vec!(seed.clone())),
        }
    }
    
    /// This method constructs a JSON string from the LocationCoordinateSchema's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the LocationCoordinatesSchema struct to a string.")?)
    } //end try_to_json
} // end LocationCoordinatesSchema

//==============================================================================
// LocationType
//==============================================================================
#[derive(Debug, PartialEq, EnumString, Display)]
#[derive(Clone, Serialize, Deserialize)]
pub enum LocationType {
    Point,
    Polygon,
}

/// Define the default value for the LocationType enum.
impl Default for LocationType {
    fn default() -> Self {
        LocationType::Point
    }
} // end LocationType

#[derive(Clone, Serialize, Deserialize)]
pub struct PointLocation {

}

#[derive(Clone, Serialize, Deserialize)]
pub struct PolygonLocation {
    #[serde(rename = "type")]
    r#type: String,
    coordinates: Vec<Vec<f32>>,
}

impl PolygonLocation {
    pub fn new(new_coordinates: Vec<Vec<f32>>) -> PolygonLocation {
        PolygonLocation {
            r#type:         String::from("Polygon"),
            coordinates:    new_coordinates
        }
    }

    pub fn test(seed: f32) -> PolygonLocation {
        PolygonLocation {
            r#type:         String::from("Polygon"),
            coordinates:    vec!(vec!(seed.clone())),
        }
    }

    pub fn world_coordinates() -> Vec<Vec<f32>> {
        vec!(
            vec!(90.0, 180.0),
            vec!(90.0, -180.0),
            vec!(-90.0, -180.0),
            vec!(-90.0, 180.0),
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LocationTypes {
    Point { location: PointLocation },
    Polygon { location: PolygonLocation },
}

//==============================================================================
// LocationSchema
//==============================================================================
/// The Location struct represent a particular geographic location relevant
/// to a particular chat message.
#[derive(Clone, Serialize, Deserialize)]
pub struct LocationSchema {
    pub aoi:    LocationTypes,

    #[serde(rename = "type")]
    pub r#type: LocationType
}

impl fmt::Display for LocationSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl LocationSchema {
    pub fn init(
        coord_value:    f32,
        new_type:       LocationType
    ) -> LocationSchema {
        LocationSchema {
            aoi:    LocationTypes::Polygon { location: PolygonLocation::test(coord_value) },
            r#type: new_type
        }
    }


    pub fn new_polygon() -> LocationSchema {
        LocationSchema {
            r#type: LocationType::Polygon,
            aoi:    LocationTypes::Polygon {
                location: PolygonLocation::new(
                    PolygonLocation::world_coordinates()
                )
            }
        }
    }

    pub fn test(seed: f32) -> LocationSchema {
        LocationSchema {
            aoi:    LocationTypes::Polygon { location: PolygonLocation::test(seed) },
            r#type: LocationType::Point,
        }
    }
    
    /// This method constructs a JSON string from the LocationSchema's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the LocationSchema struct to a string.")?)
    }
} // end LocationSchema

//==============================================================================
// struct RegionSchema
//==============================================================================
/// The Region struct describes a notable geographic area with identifying
/// information.
#[derive(Clone, Serialize, Deserialize)]
pub struct RegionSchema {
    pub abbreviation:   String,
    pub bounds:         Vec<f32>,
    pub description:    String,
    pub name:           String,

    #[serde(rename = "regionType")]
    pub region_type:    String,
}

impl fmt::Display for RegionSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl RegionSchema {
    pub fn new_test(seed: f32) -> RegionSchema {
            RegionSchema {
                abbreviation:   String::from("us"),
                bounds:         vec!(seed),
                description:    String::from(format!(
                                    "This region {} is for testing.",
                                    seed)),
                name:           String::from(format!("Test region {}", seed)),
                region_type:    String::from("Country")
            }
        }

    pub fn test(source: String, seed: f32) -> RegionSchema {
        RegionSchema {
            abbreviation:   source.clone(),
            bounds:         vec!(seed),
            description:    source.clone(),
            name:           source.clone(),
            region_type:    source.clone(),
        }
    }
    
    /// This method constructs a JSON string from the RegionSchema's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the RegionSchema struct to a string.")?)

    }
} // end RegionSchema

//==============================================================================
// struct GeoTagSchema
//==============================================================================
/// The GeoTag struct allows context information to be added to a chat message.
#[derive(Clone, Serialize, Deserialize)]
pub struct GeoTagSchema {
    #[serde(rename = "anchorEnd")]
    pub anchor_end:     i64,

    #[serde(rename = "anchorStart")]
    pub anchor_start:   i64,
    
    #[serde(rename = "anchorText")]
    pub anchor_text:    String,
    pub confidence:     f32,
    pub location:       LocationSchema,
    pub regions:        Vec<RegionSchema>,
    pub r#type:         String
}

impl fmt::Display for GeoTagSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl GeoTagSchema {
    pub fn test(source: String, seed: f32) -> GeoTagSchema {
        GeoTagSchema {
            anchor_end:     0,
            anchor_start:   0,
            anchor_text:    String::from(source.clone()),
            confidence:     0.0,
            location:       LocationSchema::test(seed),
            regions:        vec!(RegionSchema::test(source.clone(), seed)),
            r#type:         String::from(source),
        }
    }

    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the GeoTagSchema struct to a string.")?)
    }
} // end GeoTagSchema

// =============================================================================
// struct KeywordFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct KeywordFilter {
    pub query: String
}

/// Implement the trait fmt::Display for the struct KeywordFilter
/// so that these structs can be easily printed to consoles.
impl fmt::Display for KeywordFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl KeywordFilter {

    /// This method constructs a JSON string from the KeywordFilter's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the KeywordFilter struct to a string.")?)
    }
} // end KeywordFilter

// =============================================================================
// MentionType
// =============================================================================
#[derive(Serialize, Deserialize)]
pub enum MentionType {
    USER,
}

// =============================================================================
// Mention
// =============================================================================
/// This struct contains fields for searching for chat messages that
/// contain identifiers of mentioned users.
#[derive(Serialize, Deserialize)]
pub struct Mention {
    #[serde(rename = "mentionType")]
    pub mention_type:   MentionType,
    pub value:          String,
}

// =============================================================================
// MentionFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct MentionFilter {
    pub mentions:   Vec<Mention>,
}

// =============================================================================
// DomainFilterProperties
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct DomainFilterProperties {
    pub properties: Vec<String>,
}

// =============================================================================
// DomainFilterDetail
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct DomainFilterDetail  {
    // This field is a map of Domain IDs to an array of room names
    // or sender names.
    pub domains: HashMap<String, DomainFilterProperties>,
}

// =============================================================================
// SortDirection
// =============================================================================
#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortDirection {
    #[strum(serialize = "ASC")]
    ASC,
    #[strum(serialize = "DESC")]
    DESC,
}

// =============================================================================
// SortField
// =============================================================================
#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortField {
    #[strum(serialize = "DOMAIN")]
    DOMAIN,
    #[strum(serialize = "RELEVANCE")]
    RELEVANCE,
    #[strum(serialize = "ROOM")]
    ROOM,
    #[strum(serialize = "SENDER")]
    SENDER,
    #[strum(serialize = "TIME")]
    TIME,
}

// =============================================================================
// SortFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct SortFilter {
    pub orders: Vec<(SortDirection, SortField)>,
}

// =============================================================================
// ThreadIdFilter
// =============================================================================
/// This struct contains fields for filtering chat message searches
/// based on the message thread those messages belong to.
#[derive(Serialize, Deserialize)]
pub struct ThreadIdFilter {
    #[serde(rename = "threadIds")]
    pub thread_ids: Vec<String>,
}

// =============================================================================
// TimeFilterRequest
// =============================================================================
/// This struct contains fields that can be used as filters when searching
/// for chat messages within a ChatSurfer chat room.
/// 
/// Each field in this struct is considered an optional parameter from
/// ChatSurfer's perspective.  So when determining the validity of a search
/// request, these fields should be allowed to be ignored.
#[derive(Serialize, Deserialize)]
pub struct TimeFilterRequest {
    #[serde(rename = "endDateTime")]
    end_date_time:      Option<String>, //This string needs to be in DateTime format.

    #[serde(rename = "lookBackDuration")]
    look_back_duration: Option<String>,
    
    #[serde(rename = "startDateTime")]
    start_date_time:    Option<String>, //This string needs to be in DateTime format.
}

impl Default for TimeFilterRequest {
    fn default() -> Self {
        TimeFilterRequest {
            end_date_time:      Some(String::new()),
            look_back_duration: Some(String::new()),
            start_date_time:    Some(String::new()),
        }
    }
}

/// Implement the trait fmt::Display for the struct TimeFilterRequest
/// so that these structs can be easily printed to consoles.
impl fmt::Display for TimeFilterRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_string = match self.try_to_json() {
            Ok(string) => string,
            Err(e) => e.to_string()
        };

        write!(f, "{}", display_string)
    }
}

impl TimeFilterRequest {
    
    /// This method constructs a JSON string from the TimeFilterRequest's
    /// fields.
    pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(self)
            .context("Unable to convert the TimeFilterRequest struct to a string.")?)
    }
}

// =============================================================================
// UserIdFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct UserIdFilter {
    #[serde(rename = "userIds")]
    pub user_ids:    Vec<String>,
}

// =============================================================================
// TimeFilterResponse
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct TimeFilterResponse {
    #[serde(rename = "endDateTime")]
    pub end_date_time:  String,
}
























// use anyhow::{
//     Context,
//     Result,
// };

// use http::StatusCode;
// use serde::{ Deserialize, Serialize };
// use std::{
//     collections::HashMap,
//     fmt
// };
// use strum_macros::{ EnumString, Display };
// use tracing::{ event, Level };
// use uuid::Uuid;

// const MAX_ERROR_ARGUMENTS: usize = 1;
// const COORDINATES_IN_POINT: usize = 2;
// const POINTS_IN_POLYGON: usize = 4;
// pub const MAX_REGIONS: usize = 1;
// pub const MAX_REGION_BOUNDS: usize = 4;
// pub const MAX_MESSAGE_GEOTAGS: usize = 1;

// // Classification strings
// pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";

// #[derive(Debug)]
// pub struct CommonError {
//     pub message: String,
// }

// impl std::fmt::Display for CommonError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.message)
//     }
// }

// impl std::error::Error for CommonError {}

// impl CommonError {
//     pub fn from(new_message: &str) -> CommonError {
//         CommonError {
//             message: String::from(new_message),
//         }
//     }

//     pub fn from_string(new_message: String) -> CommonError {
//         CommonError {
//             message: new_message,
//         }
//     }
// }

// // #############################################################################
// // #############################################################################
// //                              Error Messages
// // #############################################################################
// // #############################################################################

// //==============================================================================
// // ErrorCode400
// //==============================================================================

// /// This structure represents an HTTP 400 Bad Request message received
// /// from ChatSurfer.
// #[derive(Serialize, Deserialize)]
// pub struct ErrorCode400 {
//     pub classification: String,
//     pub code:           u16,
    
//     #[serde(rename = "fieldErrors")]
//     pub field_errors:   Vec<FieldErrorSchema>,
//     pub message:        String,
// }

// impl Default for ErrorCode400 {
//     fn default() -> Self {
//         ErrorCode400 {
//             classification: String::from(UNCLASSIFIED_STRING),
//             code:           400,
//             field_errors:   Vec::new(),
//             message:        String::from("Bad Request"),
//         }
//     }
// }

// /// Implement the trait fmt::Display for the struct ErrorCode400
// /// so that these structs can be easily printed to consoles.
// impl fmt::Display for ErrorCode400 {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

//         write!(f, "{}", self.try_to_json())
//     }
// }

// impl ErrorCode400 {
//     /*
//      * This method constructs a JSON string from the
//      * ErrorCode400's fields.
//      */
//     pub fn try_to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

// // #############################################################################
// // #############################################################################
// //                              API Key Messages
// // #############################################################################
// // #############################################################################

// #[derive(Serialize, Deserialize)]
// #[derive(Debug, PartialEq, EnumString, Display)]
// pub enum ApiKeyStatus {
//     #[strum(serialize = "ACTIVE")]
//     ACTIVE,
//     #[strum(serialize = "DISABLED")]
//     DISABLED,
//     #[strum(serialize = "PENDING")]
//     PENDING,
// }

// #[derive(Serialize, Deserialize)]
// pub struct GetApiResponse {
//     pub classification: String,
    
//     // The Distinguished Name of the certificate used to
//     // create the API key.
//     pub dn:             String,
//     pub email:          String,
//     pub key:            String,

//     // The status of the API Key.
//     pub status:         ApiKeyStatus,
// }

// /// This enumeration defines the types of responses we can receive
// /// from ChatSurfer.
// /// 
// /// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20API%20Key>
// /// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Send%20Chat%20Message>
// /// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Get%20Chat%20Messages%20By%20Room>
// /// <https://chatsurfer.nro.mil/apidocs#operation/(U)%20Search%20Chat%20Messages>
// pub enum ChatSurferResponseType {
//     GetApiKey           { body: GetApiResponse },
//     SendChatMessage,
//     GetChatMessages     { body: GetChatMessagesResponse },
//     SearchChatMessages  { body: SearchChatMessagesResponse },
//     Failure400          { body: ErrorCode400 },
//     Failure404          { body: ErrorCode404 },
//     Failure429,
// }

// // #############################################################################
// // #############################################################################
// //                           Supporting Structures
// // #############################################################################
// // #############################################################################
// //==============================================================================
// // struct ChatMessageSchema
// //==============================================================================

// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct ChatMessageSchema {
//     pub classification: String,
//     pub domainId:       String,
//     pub geoTags:        Option<Vec<GeoTagSchema>>,
//     pub id:             Uuid,
//     pub roomName:       String,
//     pub sender:         String,
//     pub text:           String,
//     pub threadId:       Option<String>,
//     pub timestamp:      String,
//     pub userId:         Uuid,
//     pub private:        bool,
// }

// impl fmt::Display for ChatMessageSchema {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl ChatMessageSchema {
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct FieldErrorSchema {
//     pub fieldName:          String,
//     pub message:            String,
//     pub messageArguments:   [String; MAX_ERROR_ARGUMENTS],
//     pub messageCode:        String,
//     pub rejectedValue:      String
// }

// impl Default for FieldErrorSchema {
//     fn default() -> Self {
//         FieldErrorSchema {
//             fieldName:          String::new(),
//             message:            String::new(),
//             messageArguments:   [String::new()],
//             messageCode:        String::new(),
//             rejectedValue:      String::new(),
//         }
//     }
// }

// // =============================================================================
// // struct KeywordFilter
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// pub struct KeywordFilter {
//     pub query: String
// }

// /*
//  * Implement the trait fmt::Display for the struct KeywordFilter
//  * so that these structs can be easily printed to consoles.
//  */
// impl fmt::Display for KeywordFilter {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl KeywordFilter {
//     /*
//      * This method constructs a JSON string from the KeywordFilter's
//      * fields.
//      */
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// } // end KeywordFilter

// /// This enum lists the possible values for a Domain's network ID.
// #[derive(Debug, PartialEq, EnumString, Display)]
// pub enum NetworkId {
//     #[strum(serialize = "bices")]
//     Bices,

//     #[strum(serialize = "cxk")]
//     Cxk,

//     #[strum(serialize = "sipr")]
//     Sipr,

//     #[strum(serialize = "jwics")]
//     Jwics,

//     #[strum(serialize = "unclass")]
//     Unclass,
// }

// #[allow(non_camel_case_types)]
// #[derive(Debug, PartialEq, EnumString, Display)]
// pub enum JoinStatus {
//     #[strum(serialize = "JOINED")]
//     JOINED,

//     #[strum(serialize = "NOT_JOINED")]
//     NOT_JOINED,
// }

// //==============================================================================
// // struct LocationCoordinatesSchema
// //==============================================================================

// /// The LocationCoordinates union is used for the "coordinates" field in the
// /// "Location" struct to represent either a single geographic point, or a
// /// set of points to define a polygon.
// #[repr(C, packed)]
// #[derive(Serialize, Deserialize)]
// pub struct LocationCoordinatesSchema {
//     #[serde(skip)]
//     r#type:                 LocationType,

//     // The first entry represents the coordinates for a single point.
//     point_coordinates:      [f32; COORDINATES_IN_POINT],
    
//     // The second entry represents a set of points for a polygon.
//     polygon_coordinates:    [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON],

// }

// impl fmt::Display for LocationCoordinatesSchema {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl LocationCoordinatesSchema {
    
//     pub fn new_point(seed: f32) -> [f32; COORDINATES_IN_POINT] {
//         let point: [f32; COORDINATES_IN_POINT] = [seed; COORDINATES_IN_POINT];
    
//         // Return the newly constructed point.
//         point
//     }

//     pub fn new_polygon(seed: f32) -> [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON] {
//         let polygon: [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON] = [[seed; COORDINATES_IN_POINT]; POINTS_IN_POLYGON];

//         // Return the newly constructed polygon.
//         polygon
//     }

//     pub fn init(seed: f32, r#type: &LocationType) -> LocationCoordinatesSchema {
//         match r#type {
//             LocationType::Point => {
//                 LocationCoordinatesSchema {
//                     r#type: LocationType::Point,

//                     point_coordinates: LocationCoordinatesSchema::new_point(seed),

//                     // Zeroize the alternate coordinate structure.
//                     polygon_coordinates: [[0.0; COORDINATES_IN_POINT]; POINTS_IN_POLYGON]
//                 }
//             }
//             LocationType::Polygon => {
//                 LocationCoordinatesSchema {
//                     r#type: LocationType::Polygon,

//                     polygon_coordinates: LocationCoordinatesSchema::new_polygon(seed),
                    
//                     // Zeroize the alternate coordinate structure.
//                     point_coordinates: [0.0; COORDINATES_IN_POINT]
//                 }
//             }
//         }
//     } //end new

//     /*
//      * This method constructs a JSON string from the LocationCoordinateSchema's
//      * fields.
//      */
//     pub fn to_json(&self) -> String {
//         let mut point_index: usize = 0;
//         let mut polygon_index: usize = 0;
//         let mut value: f32;
//         let mut value_string: String;
//         let mut point_string: String;
//         let mut polygon_string: String;
//         let json_string: String;

//         //======================================================================
//         // Format point_coordinates field.

//         // In order to get the commas correct, we need to handle the first
//         // element specially.
//         value = self.point_coordinates[point_index];
//         value_string = format!("{:.2}", value);

//         point_string = format!("{}", value_string);
//         point_index += 1;

//         // Concatenate the point values into one string.
//         while point_index < COORDINATES_IN_POINT {
//             value = self.point_coordinates[point_index];
//             value_string = format!("{:.2}", value);

//             point_string = format!("{},{}", point_string, value_string);

//             point_index += 1;
//         } //end point loop

//         // Apply the initial JSON formatting for the point_coordinates field
//         // string.
//         json_string = format!("{{\"point_coordinates\":[{}],", point_string);

//         //======================================================================
//         // Format polygon_coordinates field.

//         point_index = 0;

//         // In order to get the commas correct, we need to handle the first array
//         // specially.
//         value = self.polygon_coordinates[polygon_index][point_index];
//         value_string = format!("{:.2}", value);

//         point_string = format!("{}", value_string);
//         point_index += 1;

//         // Concatenate the point values into one string.
//         while point_index < COORDINATES_IN_POINT {
//             value = self.polygon_coordinates[polygon_index][point_index];
//             value_string = format!("{:.2}", value);

//             point_string = format!("{},{}", point_string, value_string);
//             point_index += 1;
//         } //end point loop

//         polygon_string = format!("[{}]", point_string);
//         point_index = 0;
//         polygon_index += 1;

//         // For each point in the polygon...
//         while polygon_index < POINTS_IN_POLYGON {
//             // In order to get the commas correct, we need to handle the first array
//             // specially.
//             value = self.polygon_coordinates[polygon_index][point_index];
//             value_string = format!("{:.2}", value);

//             point_string = format!("{}", value_string);
//             point_index += 1;

//             // Concatenate the point values into one string.
//             while point_index < COORDINATES_IN_POINT {
//                 value = self.polygon_coordinates[polygon_index][point_index];
//                 value_string = format!("{:.2}", value);

//                 point_string = format!("{},{}", point_string, value_string);

//                 point_index += 1;
//             } //end point loop

//             polygon_string = format!("{},[{}]", polygon_string, point_string);
//             point_index = 0;
//             polygon_index += 1;
//         } //end polygon loop

//         // Complete the JSON formatting now that we constructed the string
//         // for the polygon_coordinates field.
//         format!("{}\"polygon_coordinates\":[{}]}}", json_string, polygon_string)
//     } //end to_json
// }

// #[derive(Debug, PartialEq, EnumString, Display)]
// #[derive(Serialize, Deserialize)]
// pub enum LocationType {
//     #[strum(serialize = "Point")]
//     Point,

//     #[strum(serialize = "Polygon")]
//     Polygon,
// }

// /*
//  * Define the default value for the LocationType enum.
//  */
// impl Default for LocationType {
//     fn default() -> Self { LocationType::Point }
// }

// #[derive(Clone, Serialize, Deserialize)]
// pub struct PointLocation {

// }

// #[derive(Clone, Serialize, Deserialize)]
// pub struct PolygonLocation {
//     #[serde(rename = "type")]
//     r#type: String,
//     coordinates: Vec<Vec<f32>>,
// }

// impl PolygonLocation {
//     pub fn new(new_coordinates: Vec<Vec<f32>>) -> PolygonLocation {
//         PolygonLocation {
//             r#type:         String::from("Polygon"),
//             coordinates:    new_coordinates
//         }
//     }

//     pub fn test(seed: f32) -> PolygonLocation {
//         PolygonLocation {
//             r#type:         String::from("Polygon"),
//             coordinates:    vec!(vec!(seed.clone())),
//         }
//     }

//     pub fn world_coordinates() -> Vec<Vec<f32>> {
//         vec!(
//             vec!(90.0, 180.0),
//             vec!(90.0, -180.0),
//             vec!(-90.0, -180.0),
//             vec!(-90.0, 180.0),
//         )
//     }
// }

// #[derive(Clone, Serialize, Deserialize)]
// pub enum LocationTypes {
//     Point { location: PointLocation },
//     Polygon { location: PolygonLocation },
// }

// //==============================================================================
// // LocationSchema
// //==============================================================================
// /// The Location struct represent a particular geographic location relevant
// /// to a particular chat message.
// #[derive(Clone, Serialize, Deserialize)]
// pub struct LocationSchema {
//     pub coordinates:    LocationCoordinatesSchema,
//     pub r#type:         LocationType
// }

// impl fmt::Display for LocationSchema {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl LocationSchema {
//     pub fn init
//     (
//         coord_value:    f32,
//         new_type:       LocationType
//     ) -> LocationSchema {
//         LocationSchema {
//             coordinates:    LocationCoordinatesSchema::init(coord_value, &new_type),
//             r#type:         new_type
//         }
//     }

//     /*
//      * This method constructs a JSON string from the LocationSchema's
//      * fields.
//      */
//     pub fn to_json(&self) -> String {
//         format!("{{\"coordinates\":{},\"type\":{}}}", self.coordinates, self.r#type)
//     }
// }

// //==============================================================================
// // struct RegionSchema
// //==============================================================================
// /// The Region struct describes a notable geographic area with identifying
// /// information.
// #[derive(Clone, Serialize, Deserialize)]
// pub struct RegionSchema {
//     pub abbreviation:   String,
//     pub bounds:         Vec<f32>,
//     pub description:    String,
//     pub name:           String,

//     #[serde(rename = "regionType")]
//     pub region_type:    String,
// }

// impl fmt::Display for RegionSchema {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl RegionSchema {
//     /*
//      * This method constructs a new RegionSchema object for testing using the
//      * given floating point value as an initial value.
//      */
//     pub fn new_test(seed: f32) -> RegionSchema {
//         RegionSchema {
//             abbreviation:   String::from("us"),
//             bounds:         [seed; MAX_REGION_BOUNDS],
//             description:    String::from(format!(
//                                 "This region {} is for testing.",
//                                 seed)),
//             name:           String::from(format!("Test region {}", seed)),
//             regionType:     String::from("Country")
//         }
//     }

//     /*
//      * This method constructs a JSON string from the RegionSchema's
//      * fields.
//      */
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

// //==============================================================================
// // struct GeoTagSchema
// //==============================================================================
// /// The GeoTag struct allows context information to be added to a chat message.
// #[derive(Clone, Serialize, Deserialize)]
// pub struct GeoTagSchema {
//     #[serde(rename = "anchorEnd")]
//     pub anchor_end:     i64,

//     #[serde(rename = "anchorStart")]
//     pub anchor_start:   i64,
    
//     #[serde(rename = "anchorText")]
//     pub anchor_text:    String,
//     pub confidence:     f32,
//     pub location:       LocationSchema,
//     pub regions:        Vec<RegionSchema>,
//     pub r#type:         String
// }

// impl fmt::Display for GeoTagSchema {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl GeoTagSchema {
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }



// // =============================================================================
// // struct GetChatMessagesResponse
// // =============================================================================

// #[derive(Serialize, Deserialize)]
// pub struct GetChatMessagesResponse {
//     pub classification: String,
//     pub messages:       Vec<ChatMessageSchema>
// }

// impl fmt::Display for GetChatMessagesResponse {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl GetChatMessagesResponse {
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

// // #############################################################################
// // #############################################################################
// //                          Search Chat Messages Data
// // #############################################################################
// // #############################################################################

// // =============================================================================
// // MentionType
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// pub enum MentionType {
//     USER,
// }

// // =============================================================================
// // Mention
// // =============================================================================
// /// This struct contains fields for searching for chat messages that
// /// contain identifiers of mentioned users.
// #[derive(Serialize, Deserialize)]
// pub struct Mention {
//     #[serde(rename = "mentionType")]
//     pub mention_type:   MentionType,
//     pub value:          String,
// }

// // =============================================================================
// // MentionFilter
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// pub struct MentionFilter {
//     pub mentions:   Vec<Mention>,
// }

// // =============================================================================
// // DomainFilterProperties
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// pub struct DomainFilterProperties {
//     pub properties: Vec<String>,
// }

// // =============================================================================
// // DomainFilterDetail
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// pub struct DomainFilterDetail  {
//     // This field is a map of Domain IDs to an array of room names
//     // or sender names.
//     pub domains: HashMap<String, DomainFilterProperties>,
// }

// impl DomainFilterDetail  {
//     // pub fn add_domain
//     // (
//     //     &self,
//     //     domainId:   &str,
//     //     names:      Vec<String>
//     // ) {
//     //     self.domains.insert(String::from(domainId), names);
//     // }

//     // pub fn new() -> RoomFilter {
//     //     RoomFilter {
//     //         domains:    HashMap::new(),
//     //     }
//     // }
// }

// // =============================================================================
// // SortDirection
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// #[derive(Debug, PartialEq, EnumString, Display)]
// pub enum SortDirection {
//     #[strum(serialize = "ASC")]
//     ASC,
//     #[strum(serialize = "DESC")]
//     DESC,
// }

// // =============================================================================
// // SortField
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// #[derive(Debug, PartialEq, EnumString, Display)]
// pub enum SortField {
//     #[strum(serialize = "DOMAIN")]
//     DOMAIN,
//     #[strum(serialize = "RELEVANCE")]
//     RELEVANCE,
//     #[strum(serialize = "ROOM")]
//     ROOM,
//     #[strum(serialize = "SENDER")]
//     SENDER,
//     #[strum(serialize = "TIME")]
//     TIME,
// }

// // =============================================================================
// // SortFilter
// // =============================================================================
// #[derive(Serialize, Deserialize)]
// pub struct SortFilter {
//     pub orders: Vec<(SortDirection, SortField)>,
// }

// // =============================================================================
// // struct ThreadIdFilter
// // =============================================================================
// /*
//  * This struct contains fields for filtering chat message searches
//  * based on the message thread those messages belong to.
//  * 
//  * We allow non-snake case names so that these fields can match those
//  * in the ChatSurfer API.
//  */
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct ThreadIdFilter {
//     pub threadIds:  Vec<String>,
// }

// // =============================================================================
// // struct TimeFilterRequest
// // =============================================================================
// /*
//  * This struct contains fields that can be used as filters when searching
//  * for chat messages within a ChatSurfer chat room.
//  * 
//  * Each field in this struct is considered an optional parameter from
//  * ChatSurfer's perspective.  So when determining the validity of a search
//  * request, these fields should be allowed to be ignored.
//  * 
//  * We allow non-snake case names so that these fields can match those
//  * in the ChatSurfer API.
//  */
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct TimeFilterRequest {
//     endDateTime:        Option<String>, //This string needs to be in DateTime format.
//     lookBackDuration:   Option<String>,
//     startDateTime:      Option<String>, //This string needs to be in DateTime format.
// }

// impl Default for TimeFilterRequest {
//     fn default() -> Self {
//         TimeFilterRequest {
//             endDateTime:        Some(String::new()),
//             lookBackDuration:   Some(String::new()),
//             startDateTime:      Some(String::new()),
//         }
//     }
// }

// /*
//  * Implement the trait fmt::Display for the struct TimeFilterRequest
//  * so that these structs can be easily printed to consoles.
//  */
// impl fmt::Display for TimeFilterRequest {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let display_string = match self.try_to_json() {
//             Ok(string) => string,
//             Err(e) => e.to_string()
//         };

//         write!(f, "{}", display_string)
//     }
// }

// impl TimeFilterRequest {
    
//     /// This method constructs a JSON string from the TimeFilterRequest's
//     /// fields.
//     pub fn try_to_json(&self) -> Result<String, anyhow::Error> {
//         Ok(serde_json::to_string(self)
//             .context("Unable to convert the TimeFilterRequest struct to a string.")?)
//     }
// }

// // =============================================================================
// // struct UserIdFilter
// // =============================================================================

// // We allow non-snake case names so that these fields can match those
// // in the ChatSurfer API.
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct UserIdFilter {
//     pub userIds:    Vec<String>,
// }

// // =============================================================================
// // struct SearchChatMessagesRequest
// // =============================================================================

// // We allow non-snake case names so that these fields can match those
// // in the ChatSurfer API.
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct SearchChatMessagesRequest {
//     pub cursor:             Option<String>,
//     pub filesOnly:          Option<bool>,
//     pub highlightResults:   Option<bool>,
//     pub keywordFilter:      Option<KeywordFilter>,
//     pub limit:              Option<i32>,
//     pub location:           Option<LocationCoordinatesSchema>,
//     pub locationFilter:     Option<bool>,
//     pub mentionFilter:      Option<MentionFilter>,
//     pub requestGeoTags:     Option<bool>,
//     pub roomFilter:         Option<DomainFilterDetail>,
//     pub senderFilter:       Option<DomainFilterDetail>,
//     pub sort:               Option<SortFilter>,
//     pub threadIdFilter:     Option<ThreadIdFilter>,
//     pub timeFilter:         Option<TimeFilterRequest>,
//     pub userIdFilter:       Option<UserIdFilter>,
// }

// impl Default for SearchChatMessagesRequest {
//     fn default() -> Self {
//         SearchChatMessagesRequest {
//             cursor:             None,
//             filesOnly:          None,
//             highlightResults:   None,
//             keywordFilter:      None,
//             limit:              None,
//             location:           None,
//             locationFilter:     None,
//             mentionFilter:      None,
//             requestGeoTags:     None,
//             roomFilter:         None,
//             senderFilter:       None,
//             sort:               None,
//             threadIdFilter:     None,
//             timeFilter:         None,
//             userIdFilter:       None,
//         }
//     }
// }

// /*
//  * Implement the trait fmt::Display for the struct SearchChatMessagesRequest
//  * so that these structs can be easily printed to consoles.
//  */
// impl fmt::Display for SearchChatMessagesRequest {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl SearchChatMessagesRequest {
//     pub fn from_string(json: String) -> SearchChatMessagesRequest {
//         serde_json::from_str(&json.as_str()).unwrap()
//     }

//     /*
//      * This method constructs a JSON string from the SearchChatMessagesRequest's
//      * fields.
//      */
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

// #[derive(Serialize, Deserialize)]
// pub enum SearchChatMessagesResponseTypes {
//     Success200 { status_code: u16, body: SearchChatMessagesResponse },
//     Failure400 { status_code: u16, error: ErrorCode400 },
//     Failure429 { status_code: u16 }
// }

// // =============================================================================
// // TimeFilterResponse
// // =============================================================================

// // We allow non-snake case names so that these fields can match those
// // in the ChatSurfer API.
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct TimeFilterResponse {
//     pub endDateTime:    String,
// }

// // =============================================================================
// // struct SearchChatMessagesResponse
// // =============================================================================

// // We allow non-snake case names so that these fields can match those
// // in the ChatSurfer API.
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct SearchChatMessagesResponse {
//     pub classification:     String,
//     pub messages:           Option<Vec<ChatMessageSchema>>,
//     pub nextCursorMark:     Option<String>,
//     pub searchTimeFiler:    TimeFilterResponse,
//     pub total:              i32,
// }

// /*
//  * Implement the trait fmt::Display for the struct SearchChatMessagesResponse
//  * so that these structs can be easily printed to consoles.
//  */
// impl fmt::Display for SearchChatMessagesResponse {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl SearchChatMessagesResponse {
//     /*
//      * This method constructs a JSON string from the
//      * SearchChatMessagesResponse's fields.
//      */
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

// // =============================================================================
// // struct SendChatMessageRequest
// // =============================================================================

// // We allow non-snake case names so that these fields can match those
// // in the ChatSurfer API.
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize)]
// pub struct SendChatMessageRequest {
//     pub classification: String,
//     pub domainId:       String,
//     pub message:        String,
//     pub nickname:       String,
//     pub roomName:       String
// }

// /*
//  * Implement the trait Default for the struct SendChatMessageRequest
//  * so that we can fall back on default values.
//  */
// impl Default for SendChatMessageRequest {
//     fn default() -> SendChatMessageRequest {
//         SendChatMessageRequest {
//             classification: String::from(UNCLASSIFIED_STRING),
//             domainId:       String::new(),
//             message:        String::new(),
//             nickname:       String::from("Edge View"),
//             roomName:       String::new()
//         }
//     }
// }

// /*
//  * Implement the trait fmt::Display for the struct SendChatMessageRequest
//  * so that these structs can be easily printed to consoles.
//  */
// impl fmt::Display for SendChatMessageRequest {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.to_json())
//     }
// }

// impl SendChatMessageRequest {
//     pub fn from_string(json: String) -> SendChatMessageRequest {
//         serde_json::from_str(&json.as_str()).unwrap()
//     }

//     /*
//      * This method constructs a JSON string from the
//      * SendChatMessageRequest's fields.
//      */
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// } //end SendChatMessageRequest

// #[derive(Serialize, Deserialize)]
// pub enum CreateMessageResponse {
//     Success204 { status_code: u16 },
//     Failure400 { error: ErrorCode400 },
//     Failure429 { status_code: u16 }
// }

// impl fmt::Display for CreateMessageResponse {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             CreateMessageResponse::Success204 { status_code } => write!(f, "{}", status_code),
//             CreateMessageResponse::Failure400 { error } => {
//                 write!(f, "{}", error)
//             },
//             CreateMessageResponse::Failure429 { status_code } => {
//                 write!(f, "{}", status_code)
//             }
//         }
//     }
// }