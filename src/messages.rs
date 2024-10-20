use serde::{ Deserialize, Serialize };
use std::{
    collections::HashMap,
    fmt,
};
use strum_macros::{ EnumString, Display };
use uuid::Uuid;

const MAX_ERROR_ARGUMENTS: usize = 1;
const COORDINATES_IN_POINT: usize = 2;
const POINTS_IN_POLYGON: usize = 4;
pub const MAX_REGIONS: usize = 1;
pub const MAX_REGION_BOUNDS: usize = 4;
pub const MAX_MESSAGE_GEOTAGS: usize = 1;

// Classification strings
pub const UNCLASSIFIED_STRING: &str = "UNCLASSIFIED";

// =============================================================================
// Error Messages

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct FieldErrorSchema {
    pub fieldName:          String,
    pub message:            String,
    pub messageArguments:   [String; MAX_ERROR_ARGUMENTS],
    pub messageCode:        String,
    pub rejectedValue:      String
}

impl Default for FieldErrorSchema {
    fn default() -> Self {
        FieldErrorSchema {
            fieldName:          String::new(),
            message:            String::new(),
            messageArguments:   [String::new()],
            messageCode:        String::new(),
            rejectedValue:      String::new(),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ErrorCode400 {
    pub classification: String,
    pub code:           i32,
    pub fieldErrors:    Vec<FieldErrorSchema>,
    pub message:        String
}

impl Default for ErrorCode400 {
    fn default() -> Self {
        ErrorCode400 {
            classification: String::from(UNCLASSIFIED_STRING),
            code:           400,
            fieldErrors:    Vec::new(),
            message:        String::from("Bad Request"),
        }
    }
}

/*
 * Implement the trait fmt::Display for the struct SearchChatMessagesResponse
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for ErrorCode400 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl ErrorCode400 {
    

    /*
     * This method constructs a JSON string from the
     * ErrorCode400's fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
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
    pub status:         ApiKeyStatus,
}

// =============================================================================
// General Messages

/// This enum lists the possible values for a Domain's network ID.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum NetworkId {
    #[strum(serialize = "bices")]
    bices,

    #[strum(serialize = "cxk")]
    cxk,

    #[strum(serialize = "sipr")]
    sipr,

    #[strum(serialize = "jwics")]
    jwics,

    #[strum(serialize = "unclass")]
    unclass,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum JoinStatus {
    #[strum(serialize = "JOINED")]
    JOINED,

    #[strum(serialize = "NOT_JOINED")]
    NOT_JOINED,
}

//==============================================================================
// struct LocationCoordinatesSchema
//==============================================================================

/// The LocationCoordinates union is used for the "coordinates" field in the
/// "Location" struct to represent either a single geographic point, or a
/// set of points to define a polygon.
#[repr(C, packed)]
#[derive(Serialize, Deserialize)]
pub struct LocationCoordinatesSchema {
    #[serde(skip)]
    r#type:                 LocationType,

    // The first entry represents the coordinates for a single point.
    point_coordinates:      [f32; COORDINATES_IN_POINT],
    
    // The second entry represents a set of points for a polygon.
    polygon_coordinates:    [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON],

}

impl fmt::Display for LocationCoordinatesSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl LocationCoordinatesSchema {
    
    pub fn new_point(seed: f32) -> [f32; COORDINATES_IN_POINT] {
        let point: [f32; COORDINATES_IN_POINT] = [seed; COORDINATES_IN_POINT];
    
        // Return the newly constructed point.
        point
    }

    pub fn new_polygon(seed: f32) -> [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON] {
        let polygon: [[f32; COORDINATES_IN_POINT]; POINTS_IN_POLYGON] = [[seed; COORDINATES_IN_POINT]; POINTS_IN_POLYGON];

        // Return the newly constructed polygon.
        polygon
    }

    pub fn init(seed: f32, r#type: &LocationType) -> LocationCoordinatesSchema {
        match r#type {
            LocationType::Point => {
                LocationCoordinatesSchema {
                    r#type: LocationType::Point,

                    point_coordinates: LocationCoordinatesSchema::new_point(seed),

                    // Zeroize the alternate coordinate structure.
                    polygon_coordinates: [[0.0; COORDINATES_IN_POINT]; POINTS_IN_POLYGON]
                }
            }
            LocationType::Polygon => {
                LocationCoordinatesSchema {
                    r#type: LocationType::Polygon,

                    polygon_coordinates: LocationCoordinatesSchema::new_polygon(seed),
                    
                    // Zeroize the alternate coordinate structure.
                    point_coordinates: [0.0; COORDINATES_IN_POINT]
                }
            }
        }
    } //end new

    /*
     * This method constructs a JSON string from the LocationCoordinateSchema's
     * fields.
     */
    pub fn to_json(&self) -> String {
        let mut point_index: usize = 0;
        let mut polygon_index: usize = 0;
        let mut value: f32;
        let mut value_string: String;
        let mut point_string: String;
        let mut polygon_string: String;
        let json_string: String;

        //======================================================================
        // Format point_coordinates field.

        // In order to get the commas correct, we need to handle the first
        // element specially.
        value = self.point_coordinates[point_index];
        value_string = format!("{:.2}", value);

        point_string = format!("{}", value_string);
        point_index += 1;

        // Concatenate the point values into one string.
        while point_index < COORDINATES_IN_POINT {
            value = self.point_coordinates[point_index];
            value_string = format!("{:.2}", value);

            point_string = format!("{},{}", point_string, value_string);

            point_index += 1;
        } //end point loop

        // Apply the initial JSON formatting for the point_coordinates field
        // string.
        json_string = format!("{{\"point_coordinates\":[{}],", point_string);

        //======================================================================
        // Format polygon_coordinates field.

        point_index = 0;

        // In order to get the commas correct, we need to handle the first array
        // specially.
        value = self.polygon_coordinates[polygon_index][point_index];
        value_string = format!("{:.2}", value);

        point_string = format!("{}", value_string);
        point_index += 1;

        // Concatenate the point values into one string.
        while point_index < COORDINATES_IN_POINT {
            value = self.polygon_coordinates[polygon_index][point_index];
            value_string = format!("{:.2}", value);

            point_string = format!("{},{}", point_string, value_string);
            point_index += 1;
        } //end point loop

        polygon_string = format!("[{}]", point_string);
        point_index = 0;
        polygon_index += 1;

        // For each point in the polygon...
        while polygon_index < POINTS_IN_POLYGON {
            // In order to get the commas correct, we need to handle the first array
            // specially.
            value = self.polygon_coordinates[polygon_index][point_index];
            value_string = format!("{:.2}", value);

            point_string = format!("{}", value_string);
            point_index += 1;

            // Concatenate the point values into one string.
            while point_index < COORDINATES_IN_POINT {
                value = self.polygon_coordinates[polygon_index][point_index];
                value_string = format!("{:.2}", value);

                point_string = format!("{},{}", point_string, value_string);

                point_index += 1;
            } //end point loop

            polygon_string = format!("{},[{}]", polygon_string, point_string);
            point_index = 0;
            polygon_index += 1;
        } //end polygon loop

        // Complete the JSON formatting now that we constructed the string
        // for the polygon_coordinates field.
        format!("{}\"polygon_coordinates\":[{}]}}", json_string, polygon_string)
    } //end to_json
}

#[derive(Debug, PartialEq, EnumString, Display)]
#[derive(Serialize, Deserialize)]
pub enum LocationType {
    #[strum(serialize = "Point")]
    Point,

    #[strum(serialize = "Polygon")]
    Polygon,
}

/*
 * Define the default value for the LocationType enum.
 */
impl Default for LocationType {
    fn default() -> Self { LocationType::Point }
}

//==============================================================================
// struct LocationSchema
//==============================================================================

/// The Location struct represent a particular geographic location relevant
/// to a particular chat message.
#[derive(Serialize, Deserialize)]
pub struct LocationSchema {
    pub coordinates:    LocationCoordinatesSchema,
    pub r#type:         LocationType
}

impl fmt::Display for LocationSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl LocationSchema {
    pub fn init
    (
        coord_value:    f32,
        new_type:       LocationType
    ) -> LocationSchema {
        LocationSchema {
            coordinates:    LocationCoordinatesSchema::init(coord_value, &new_type),
            r#type:         new_type
        }
    }

    /*
     * This method constructs a JSON string from the LocationSchema's
     * fields.
     */
    pub fn to_json(&self) -> String {
        format!("{{\"coordinates\":{},\"type\":{}}}", self.coordinates, self.r#type)
    }
}

//==============================================================================
// struct RegionSchema
//==============================================================================

/// The Region struct describes a notable geographic area with identifying
/// information.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct RegionSchema {
    pub abbreviation:   String,
    pub bounds:         [f32; MAX_REGION_BOUNDS],
    pub description:    String,
    pub name:           String,
    pub regionType:     String
}

impl fmt::Display for RegionSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl RegionSchema {
    /*
     * This method constructs a new RegionSchema object for testing using the
     * given floating point value as an initial value.
     */
    pub fn new_test(seed: f32) -> RegionSchema {
        RegionSchema {
            abbreviation:   String::from("us"),
            bounds:         [seed; MAX_REGION_BOUNDS],
            description:    String::from(format!(
                                "This region {} is for testing.",
                                seed)),
            name:           String::from(format!("Test region {}", seed)),
            regionType:     String::from("Country")
        }
    }

    /*
     * This method constructs a JSON string from the RegionSchema's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

//==============================================================================
// struct GeoTagSchema
//==============================================================================

/// The GeoTag struct allows context information to be added to a chat message.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct GeoTagSchema {
    pub anchorEnd:      i64,
    pub anchorStart:    i64,
    pub anchorText:     String,
    pub confidence:     f32,
    pub location:       LocationSchema,
    pub regions:        [RegionSchema; MAX_REGIONS],
    pub r#type:         String
}

impl fmt::Display for GeoTagSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl GeoTagSchema {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

//==============================================================================
// struct ChatMessageSchema
//==============================================================================

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ChatMessageSchema {
    pub classification: String,
    pub domainId:       String,
    pub geoTags:        [GeoTagSchema; MAX_MESSAGE_GEOTAGS],
    pub id:             Uuid,
    pub roomName:       String,
    pub sender:         String,
    pub text:           String,
    pub threadId:       Uuid,
    pub timestamp:      String,
    pub userId:         Uuid
}

impl fmt::Display for ChatMessageSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl ChatMessageSchema {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct GetChatMessagesResponse
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct GetChatMessagesResponse {
    pub classification: String,
    pub messages:       Vec<ChatMessageSchema>
}

impl fmt::Display for GetChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl GetChatMessagesResponse {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// #############################################################################
// #############################################################################
//                          Search Chat Messages Data
// #############################################################################
// #############################################################################

// =============================================================================
// struct KeywordFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct KeywordFilter {
    pub query: String
}

/*
 * Implement the trait fmt::Display for the struct KeywordFilter
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for KeywordFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl KeywordFilter {
    /*
     * This method constructs a JSON string from the KeywordFilter's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
} // end KeywordFilter

// =============================================================================
// struct MentionFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub enum MentionType {
    USER,
}

/*
 * This struct contains fields for searching for chat messages that
 * contain identifiers of mentioned users.
 * 
 * We allow non-snake case names so that these fields can match those
 * in the ChatSurfer API.
 */
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Mention {
    pub mentionType:    MentionType,
    pub value:          String,
}

#[derive(Serialize, Deserialize)]
pub struct MentionFilter {
    pub mentions:   Vec<Mention>,
}
// =============================================================================
// struct RoomFilter
// =============================================================================
#[derive(Serialize, Deserialize)]
pub struct DomainFilterProperties {
    pub properties: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DomainFilterDetail  {
    // This field is a map of Domain IDs to an array of room names
    // or sender names.
    pub domains: HashMap<String, DomainFilterProperties>,
}

impl DomainFilterDetail  {
    // pub fn add_domain
    // (
    //     &self,
    //     domainId:   &str,
    //     names:      Vec<String>
    // ) {
    //     self.domains.insert(String::from(domainId), names);
    // }

    // pub fn new() -> RoomFilter {
    //     RoomFilter {
    //         domains:    HashMap::new(),
    //     }
    // }
}

// =============================================================================
// struct SortFiler
// =============================================================================
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortDirection {
    #[strum(serialize = "ASC")]
    ASC,
    #[strum(serialize = "DESC")]
    DESC,
}

#[allow(non_camel_case_types)]
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

#[derive(Serialize, Deserialize)]
pub struct SortFilter {
    pub orders: Vec<(SortDirection, SortField)>,
}

// =============================================================================
// struct ThreadIdFilter
// =============================================================================
/*
 * This struct contains fields for filtering chat message searches
 * based on the message thread those messages belong to.
 * 
 * We allow non-snake case names so that these fields can match those
 * in the ChatSurfer API.
 */
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ThreadIdFilter {
    pub threadIds:  Vec<String>,
}

// =============================================================================
// struct TimeFilterRequest
// =============================================================================
/*
 * This struct contains fields that can be used as filters when searching
 * for chat messages within a ChatSurfer chat room.
 * 
 * Each field in this struct is considered an optional parameter from
 * ChatSurfer's perspective.  So when determining the validity of a search
 * request, these fields should be allowed to be ignored.
 * 
 * We allow non-snake case names so that these fields can match those
 * in the ChatSurfer API.
 */
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct TimeFilterRequest {
    endDateTime:        Option<String>, //This string needs to be in DateTime format.
    lookBackDuration:   Option<String>,
    startDateTime:      Option<String>, //This string needs to be in DateTime format.
}

impl Default for TimeFilterRequest {
    fn default() -> Self {
        TimeFilterRequest {
            endDateTime:        Some(String::new()),
            lookBackDuration:   Some(String::new()),
            startDateTime:      Some(String::new()),
        }
    }
}

/*
 * Implement the trait fmt::Display for the struct TimeFilterRequest
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for TimeFilterRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl TimeFilterRequest {
    /*
     * This method constructs a JSON string from the TimeFilterRequest's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct UserIdFilter
// =============================================================================

// We allow non-snake case names so that these fields can match those
// in the ChatSurfer API.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct UserIdFilter {
    pub userIds:    Vec<String>,
}

// =============================================================================
// struct SearchChatMessagesRequest
// =============================================================================

// We allow non-snake case names so that these fields can match those
// in the ChatSurfer API.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesRequest {
    pub cursor:             Option<String>,
    pub filesOnly:          Option<bool>,
    pub highlightResults:   Option<bool>,
    pub keywordFilter:      Option<KeywordFilter>,
    pub limit:              Option<i32>,
    pub location:           Option<LocationCoordinatesSchema>,
    pub locationFilter:     Option<bool>,
    pub mentionFilter:      Option<MentionFilter>,
    pub requestGeoTags:     Option<bool>,
    pub roomFilter:         Option<DomainFilterDetail>,
    pub senderFilter:       Option<DomainFilterDetail>,
    pub sort:               Option<SortFilter>,
    pub threadIdFilter:     Option<ThreadIdFilter>,
    pub timeFilter:         Option<TimeFilterRequest>,
    pub userIdFilter:       Option<UserIdFilter>,
}

impl Default for SearchChatMessagesRequest {
    fn default() -> Self {
        SearchChatMessagesRequest {
            cursor:             None,
            filesOnly:          None,
            highlightResults:   None,
            keywordFilter:      None,
            limit:              None,
            location:           None,
            locationFilter:     None,
            mentionFilter:      None,
            requestGeoTags:     None,
            roomFilter:         None,
            senderFilter:       None,
            sort:               None,
            threadIdFilter:     None,
            timeFilter:         None,
            userIdFilter:       None,
        }
    }
}

/*
 * Implement the trait fmt::Display for the struct SearchChatMessagesRequest
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for SearchChatMessagesRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SearchChatMessagesRequest {
    pub fn from_string(json: String) -> SearchChatMessagesRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }

    /*
     * This method constructs a JSON string from the SearchChatMessagesRequest's
     * fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub enum SearchChatMessagesResponseTypes {
    Success200 { status_code: u16, body: SearchChatMessagesResponse },
    Failure400 { status_code: u16, error: ErrorCode400 },
    Failure429 { status_code: u16 }
}

// =============================================================================
// struct TimeFilterResponse
// =============================================================================

// We allow non-snake case names so that these fields can match those
// in the ChatSurfer API.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct TimeFilterResponse {
    pub endDateTime:    String,
}

// =============================================================================
// struct SearchChatMessagesResponse
// =============================================================================

// We allow non-snake case names so that these fields can match those
// in the ChatSurfer API.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SearchChatMessagesResponse {
    pub classification:     String,
    pub messages:           Option<Vec<ChatMessageSchema>>,
    pub nextCursorMark:     Option<String>,
    pub searchTimeFiler:    TimeFilterResponse,
    pub total:              i32,
}

/*
 * Implement the trait fmt::Display for the struct SearchChatMessagesResponse
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for SearchChatMessagesResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SearchChatMessagesResponse {
    /*
     * This method constructs a JSON string from the
     * SearchChatMessagesResponse's fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// =============================================================================
// struct SendChatMessageRequest
// =============================================================================

// We allow non-snake case names so that these fields can match those
// in the ChatSurfer API.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct SendChatMessageRequest {
    pub classification: String,
    pub domainId:       String,
    pub message:        String,
    pub nickname:       String,
    pub roomName:       String
}

/*
 * Implement the trait Default for the struct SendChatMessageRequest
 * so that we can fall back on default values.
 */
impl Default for SendChatMessageRequest {
    fn default() -> SendChatMessageRequest {
        SendChatMessageRequest {
            classification: String::from(UNCLASSIFIED_STRING),
            domainId:       String::new(),
            message:        String::new(),
            nickname:       String::from("Edge View"),
            roomName:       String::new()
        }
    }
}

/*
 * Implement the trait fmt::Display for the struct SendChatMessageRequest
 * so that these structs can be easily printed to consoles.
 */
impl fmt::Display for SendChatMessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl SendChatMessageRequest {
    pub fn from_string(json: String) -> SendChatMessageRequest {
        serde_json::from_str(&json.as_str()).unwrap()
    }

    /*
     * This method constructs a JSON string from the
     * SendChatMessageRequest's fields.
     */
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
} //end SendChatMessageRequest

#[derive(Serialize, Deserialize)]
pub enum CreateMessageResponse {
    Success204 { status_code: u16 },
    Failure400 { error: ErrorCode400 },
    Failure429 { status_code: u16 }
}

impl fmt::Display for CreateMessageResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CreateMessageResponse::Success204 { status_code } => write!(f, "{}", status_code),
            CreateMessageResponse::Failure400 { error } => {
                write!(f, "{}", error)
            },
            CreateMessageResponse::Failure429 { status_code } => {
                write!(f, "{}", status_code)
            }
        }
    }
}