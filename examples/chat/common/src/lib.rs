use std::collections::HashMap;

use framework::BiStream;
use serde::{Deserialize, Serialize};

/// TLS certificate (self-signed for debug purposes)
pub const CERTIFICATE: &[u8] = include_bytes!("localhost.crt");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomDescription {
    pub name: String,
    pub long_desc: String,
}

pub type ChatMessage = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageMetaData {
    pub username: String,
    pub user_color: [u8; 3],

    pub msg: ChatMessage,
}

#[tarpc::service]
pub trait ChatService {
    /// Gets the rooms by name
    async fn get_rooms() -> HashMap<String, RoomDescription>;

    /// Returns true on success
    async fn create_room(name: String) -> bool;

    /// Connects to the given room
    async fn chat(
        room_name: String,
        username: String,
        user_color: [u8; 3],
    ) -> BiStream<MessageMetaData, ChatMessage>;
}
