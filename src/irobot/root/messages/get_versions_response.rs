use crate::irobot::root::Message;

pub struct GetVersionsResponse {
    pub board_id: u8,
    pub firmware_major_version: u8,
    pub firmware_minor_version: u8,
    pub hardware_major_version: u8,
    pub hardware_minor_version: u8,
    pub bootloader_major_version: u8,
    pub bootloader_minor_version: u8,
    pub protocol_major_version: u8,
    pub protocol_minor_version: u8,
    pub patch_number: u8,
}

impl GetVersionsResponse {
    pub fn new(message: Message) -> GetVersionsResponse {
        GetVersionsResponse {
            board_id: message.data[3],
            firmware_major_version: message.data[4],
            firmware_minor_version: message.data[5],
            hardware_major_version: message.data[6],
            hardware_minor_version: message.data[7],
            bootloader_major_version: message.data[8],
            bootloader_minor_version: message.data[9],
            protocol_major_version: message.data[10],
            protocol_minor_version: message.data[11],
            patch_number: message.data[12],
        }
    }
}
