use super::messages::{
    DriveArcFinishedResponse, DriveDistanceFinishedResponse, GetVersionsResponse,
    MarkerFinishedResponse, RotateAngleFinishedResponse,
};
use btleplug::api::{CharPropFlags, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use crc::{Algorithm, Crc};
use uuid::{uuid, Uuid};

use futures::stream::StreamExt;

use crate::utils::MessageStorage;

const ROOT_IDENTIFIER_UUID: Uuid = uuid!("48c5d828-ac2a-442d-97a3-0c9822b04979");
const ROOT_RX_CHARACTERISTIC: Uuid = uuid!("6e400002-b5a3-f393-e0a9-e50e24dcca9e");
const ROOT_TX_CHARACTERISTIC: Uuid = uuid!("6e400003-b5a3-f393-e0a9-e50e24dcca9e");

pub enum MarkerPosition {
    NothingDown = 0x00,
    MarkerDown = 0x01, //ErasorDown = 0x02,
}

pub enum RootDeviceId {
    General = 0x00,
    Motors = 0x01,
    Marker = 0x02,
    LEDLights = 0x03,
    Sound = 0x05,
    CliffSensor = 0x14,
}

pub enum LEDLightsState {
    Off = 0x00,
    On = 0x01,
    Blink = 0x02,
    Spin = 0x03,
}

#[derive(PartialEq, Eq, Hash)]
struct RootMessageKey {
    device: u8,
    command: u8,
    id: u8,
}

// Wrapper for the data from a bluetooth message.
pub struct Message {
    pub data: Vec<u8>,
}

// Algorithm used for calculating CRC on bluetooth messages
// See https://github.com/iRobotEducation/root-robot-ble-protocol
const ROOT_CRC_ALGORITHM: Algorithm<u8> = Algorithm {
    poly: 0x07,
    init: 0x00,
    refin: false,
    refout: false,
    xorout: 0x00,
    check: 0x00,
    residue: 0x00,
};
const ROOT_CRC: Crc<u8> = Crc::<u8>::new(&ROOT_CRC_ALGORITHM);

// The full format requires that the 20th byte is a calculated checksum.
// This takes the intended packet, pads it to the correct length,
// then calculates the CRC and adds it.
fn build_checked_packet(mut packet: Vec<u8>) -> Vec<u8> {
    // Assumes the payload is a valid unsigned payload
    assert!(packet.len() < 20);

    // pad in 0's as needed
    while packet.len() < 19 {
        packet.push(0x00);
    }

    let mut digest = ROOT_CRC.digest();
    digest.update(&packet);

    packet.push(digest.finalize());
    return packet;
}

// Root robot defines a specific service to identify it, this checks for that UUID.
pub fn is_root_robot(peripheral: &Peripheral) -> bool {
    peripheral
        .services()
        .into_iter()
        .any(|a| a.uuid == ROOT_IDENTIFIER_UUID)
}

pub struct RootRobot {
    peripheral: Peripheral,
    message_storage: MessageStorage<RootMessageKey, Message>,
}

impl RootRobot {
    pub fn new(peripheral: Peripheral) -> RootRobot {
        RootRobot {
            peripheral: peripheral,
            message_storage: MessageStorage::new(),
        }
    }

    /////////////////////////////////////////
    // Device 0 - General
    /////////////////////////////////////////

    // Command 0 - Get Versions
    // Request a response packet with Command 0 and matching ID containing the software and hardware version numbers.
    pub async fn print_versions(&self) {
        let versions_cmd = vec![0x00, 0x00, 0x10, 0xA5];
        self.send_msg(versions_cmd, WriteType::WithResponse).await;

        let version =
            GetVersionsResponse::new(self.wait_for_message(RootDeviceId::General, 0x00, 0x10));
        println!(
            "Firmware version: {}.{}",
            version.firmware_major_version, version.firmware_minor_version
        );
        println!(
            "Hardware version: {}.{}",
            version.hardware_major_version, version.hardware_minor_version
        );
        println!(
            "Bootloader version: {}.{}",
            version.bootloader_major_version, version.bootloader_minor_version
        );
        println!(
            "Protocol version: {}.{}",
            version.protocol_major_version, version.protocol_minor_version
        );
        println!("Patch number: {}", version.patch_number);
    }

    // Command 3 - Stop and Reset
    // Immediately stop the robot and cancel any pending actions. (Same as pressing the stop button in the Root Coding app.)
    pub async fn stop_and_reset(&self) {
        let stop_cmd = vec![RootDeviceId::General as u8, 0x03];
        self.send_msg(stop_cmd, WriteType::WithoutResponse).await;
    }

    /////////////////////////////////////////
    // Device 1 - Motors
    /////////////////////////////////////////

    // Command 8 - Drive Distance
    // Drive a set distance in a straight line. Robot sends a Drive Distance Finished response packet with Command 8 and matching ID when finished.
    pub async fn drive_distance(&self, distance_mm: i32) -> DriveDistanceFinishedResponse {
        let distance_bytes = distance_mm.to_be_bytes();
        let drive_cmd = vec![
            RootDeviceId::Motors as u8,
            0x08,
            0x11,
            distance_bytes[0],
            distance_bytes[1],
            distance_bytes[2],
            distance_bytes[3],
        ];
        self.send_msg(drive_cmd, WriteType::WithResponse).await;

        return DriveDistanceFinishedResponse::new(self.wait_for_message(
            RootDeviceId::Motors,
            0x08,
            0x11,
        ));
    }

    // Command 12 - Rotate Angle
    // Rotate in place by a set angle. Robot sends a Rotate Angle Finished response packet with Command 12 and matching ID when finished.
    pub async fn rotate_angle(&self, angle_deci_degrees: i32) -> RotateAngleFinishedResponse {
        let degree_bytes = angle_deci_degrees.to_be_bytes();
        let rotate_cmd = vec![
            RootDeviceId::Motors as u8,
            0x0C,
            0x12,
            degree_bytes[0],
            degree_bytes[1],
            degree_bytes[2],
            degree_bytes[3],
        ];
        self.send_msg(rotate_cmd, WriteType::WithResponse).await;
        return RotateAngleFinishedResponse::new(self.wait_for_message(
            RootDeviceId::Motors,
            0x0C,
            0x12,
        ));
    }

    // Command 15 - Reset Position
    // Reset the estimated robot location to (0, 0) and orientation to 90 degrees of yaw (pointing in the direction of
    // positive-y on a right-handed xy plane). The robot also resets the position when the Root robot nose button
    // (or Create 3 power button) is pressed, when a Stop and Reset packet is received, and when a new Bluetooth connection is made.
    pub async fn reset_position(&self) {
        let reset_cmd = vec![RootDeviceId::Motors as u8, 0xF, 0x00];
        self.send_msg(reset_cmd, WriteType::WithoutResponse).await;
    }

    // Command 27 - Drive Arc
    // Drive the length of an arc defined by a set angle and radius. Robot sends a Drive Arc Finished response packet
    // with Command 27 and matching ID when finished.
    pub async fn drive_arc(&self, angle: i32, radius: i32) {
        let angle_bytes = angle.to_be_bytes();
        let radius_bytes = radius.to_be_bytes();

        let drive_cmd = vec![
            RootDeviceId::Motors as u8,
            0x1B,
            0x1B,
            angle_bytes[0],
            angle_bytes[1],
            angle_bytes[2],
            angle_bytes[3],
            radius_bytes[0],
            radius_bytes[1],
            radius_bytes[2],
            radius_bytes[3],
        ];
        self.send_msg(drive_cmd, WriteType::WithResponse).await;

        DriveArcFinishedResponse::new(self.wait_for_message(RootDeviceId::Motors, 0x1B, 0x1B));
    }

    /////////////////////////////////////////
    // Device 2 - Marker
    /////////////////////////////////////////

    // Command - 0
    // Set the position of the marker/eraser actuator. Robot sends a Marker/Eraser Position Finished packet with Command 0 and matching ID when finished.
    pub async fn set_marker_position(&self, position: MarkerPosition) {
        let marker_cmd = vec![RootDeviceId::Marker as u8, 0x00, 0x13, position as u8];
        self.send_msg(marker_cmd, WriteType::WithResponse).await;
        MarkerFinishedResponse::new(self.wait_for_message(RootDeviceId::Marker, 0x00, 0x13));
    }

    /////////////////////////////////////////
    // Device 3 - LED Lights
    /////////////////////////////////////////

    // Command 2
    // Set LED cross animation type and color.
    pub async fn set_lights(&self, lights_state: LEDLightsState, r: u8, g: u8, b: u8) {
        let color_cmd = vec![
            RootDeviceId::LEDLights as u8,
            0x02,
            0x01,
            lights_state as u8,
            r,
            g,
            b,
        ];
        self.send_msg(color_cmd, WriteType::WithoutResponse).await;
    }

    /////////////////////////////////////////
    // Device 5 - Sound
    /////////////////////////////////////////

    // Command 4
    // Speak a text string in robot language. Robot sends a Say Phrase Finished response packet with Command 4 and matching ID when finished.
    pub async fn say_phrase(&self, phrase: &str) {
        let mut phrase_cmd = vec![RootDeviceId::Sound as u8, 0x04, 0x00];

        let phrase_bytes = phrase.bytes();
        assert!(phrase_bytes.len() < 16);

        for (_, byte) in phrase_bytes.enumerate() {
            phrase_cmd.push(byte);
        }

        self.send_msg(phrase_cmd, WriteType::WithoutResponse).await;

        // Ignore response
        self.wait_for_message(RootDeviceId::Sound, 0x04, 0x00);
    }

    // Print out chracteristics from the robot
    pub async fn print_characteristics(&self) {
        println!(
            "Discover root peripheral {:?} services...",
            self.peripheral.id()
        );
        for service in self.peripheral.services() {
            println!(
                "Service UUID {}, primary: {}",
                service.uuid, service.primary
            );
            for characteristic in service.characteristics {
                println!("  {:?}", characteristic);
            }
        }
    }

    // Notify the tx characteristic that it should send us responses
    pub async fn subscribe(&self) {
        let characteristics = self.peripheral.characteristics();
        let tx_characteristic = characteristics
            .iter()
            .find(|c| {
                c.uuid == ROOT_TX_CHARACTERISTIC && c.properties.contains(CharPropFlags::INDICATE)
            })
            .expect("Unable to find read characteristic");
        self.peripheral
            .subscribe(tx_characteristic)
            .await
            .expect("Failed to subscribe");
    }

    // Process messages in a loop
    // TODO: Cancle
    pub async fn run_message_loop(&self) {
        loop {
            let mut notification_stream = self
                .peripheral
                .notifications()
                .await
                .expect("Message")
                .take(1);

            while let Some(data) = notification_stream.next().await {
                let msk = RootMessageKey {
                    device: data.value[0],
                    command: data.value[1],
                    id: data.value[2],
                };

                //println!("Got message {}", data.value[0]);

                // Sometimes we want to immediately react to a message
                // TODO: this blocks reading new messages until its completed
                if msk.device == RootDeviceId::CliffSensor as u8 {
                    println!("Got cliff sensor message {}", data.value[7]);
                    if data.value[7] > 0 {
                        self.stop_and_reset().await;
                        panic!("Shutting down due to cliff")
                    }
                } else {
                    self.message_storage
                        .put_message(msk, Message { data: data.value });
                }
            }
        }
    }

    // Disconnects from the peripheral
    pub async fn disconnect(&self) {
        if self
            .peripheral
            .is_connected()
            .await
            .expect("Peripheral is not connected")
        {
            println!(
                "Disconnecting from peripheral {:?}...",
                self.peripheral.id()
            );
            self.peripheral
                .disconnect()
                .await
                .expect("Error disconnecting from BLE peripheral");
        }
    }

    // wait for a message to be received by the robot
    // TODO: Async
    pub fn wait_for_message(&self, device: RootDeviceId, command: u8, id: u8) -> Message {
        let msk = RootMessageKey {
            device: device as u8,
            command: command,
            id: id,
        };

        // TODO: Handle no message case
        self.message_storage.wait_for_message(msk).unwrap()
    }

    // Calculate the CRC and send the message to the robot
    pub async fn send_msg(&self, vector: Vec<u8>, write_type: WriteType) {
        let characteristics = self.peripheral.characteristics();

        // TODO: Do I need to find this each time?
        let rx_characteristic = characteristics
            .iter()
            .find(|c| c.uuid == ROOT_RX_CHARACTERISTIC)
            .expect("Unable to find write characteristic");

        if self
            .peripheral
            .is_connected()
            .await
            .expect("Failed to check if device is connected")
        {
            self.peripheral
                .write(rx_characteristic, &build_checked_packet(vector), write_type)
                .await
                .expect("Failed to send message");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_forward_crc() {
        let test_vector: Vec<u8> = vec![
            0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64,
        ];
        assert_eq!(build_checked_packet(test_vector)[19], 0xD1);
    }

    #[test]
    fn test_drive_back_crc() {
        let test_vector: Vec<u8> = vec![
            0x01, 0x04, 0x00, 0xFF, 0xFF, 0xFF, 0x9C, 0xFF, 0xFF, 0xFF, 0x9C, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(build_checked_packet(test_vector)[19], 0x71);
    }
}
