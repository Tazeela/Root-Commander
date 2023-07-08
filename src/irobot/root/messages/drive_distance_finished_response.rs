use crate::irobot::root::Message;

pub struct DriveDistanceFinishedResponse {
    pub timestamp: u32,
    pub x_coord: i32,
    pub y_coord: i32,
    pub heading: i16,
}

impl DriveDistanceFinishedResponse {
    pub fn new(message: Message) -> DriveDistanceFinishedResponse {
        DriveDistanceFinishedResponse {
            timestamp: u32::from_ne_bytes(message.data[3..7].try_into().unwrap()),
            x_coord: i32::from_ne_bytes(message.data[7..11].try_into().unwrap()),
            y_coord: i32::from_ne_bytes(message.data[11..15].try_into().unwrap()),
            heading: i16::from_ne_bytes(message.data[15..17].try_into().unwrap()),
        }
    }
}
