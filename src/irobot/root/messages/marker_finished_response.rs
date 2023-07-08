use crate::irobot::root::Message;

pub struct MarkerFinishedResponse {
    pub position: u8,
}

impl MarkerFinishedResponse {
    pub fn new(message: Message) -> MarkerFinishedResponse {
        MarkerFinishedResponse {
            position: message.data[3],
        }
    }
}
