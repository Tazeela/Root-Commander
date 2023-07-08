use std::{error::Error, sync::Arc};

mod messagestorage;
mod orchestrator;

mod irobot;
use irobot::root::{is_root_robot, LEDLightsState, RootRobot};

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use orchestrator::{LinearOrchestrator, Point};
use std::time::Duration;
use tokio::time;

// Finds the root robot by looking for the ROOT Identifier UUID
pub async fn find_root_peripheral() -> Option<RootRobot> {
    let manager = Manager::new()
        .await
        .expect("Failed to load bluetooth manager");
    let adapter_list = manager.adapters().await.expect("Failed to load adapters");
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        println!(
            "Starting scan on {}...",
            adapter
                .adapter_info()
                .await
                .expect("Failed to load adapter_info")
        );

        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await.expect("No peripherals found");

        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            for peripheral in adapter.peripherals().await.unwrap() {
                let properties = peripheral.properties().await.expect("No properties");
                let is_connected = peripheral.is_connected().await.expect("Unknown connection");
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                println!(
                    "Peripheral {:?} is connected: {:?}",
                    local_name, is_connected
                );
                if !is_connected {
                    println!("Connecting to peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                }
                let is_connected = peripheral.is_connected().await.expect("is _connected?");
                println!(
                    "Now connected ({:?}) to peripheral {:?}...",
                    is_connected, &local_name
                );
                peripheral
                    .discover_services()
                    .await
                    .expect("Failed to discover services");

                if is_root_robot(&peripheral) {
                    return Some(RootRobot::new(peripheral));
                }
            }
        }
    }

    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get a coonection to the device
    // Note that we store it in an arc so that it can be shared to the message loop
    let root_peripheral: Arc<RootRobot> = Arc::new(
        find_root_peripheral()
            .await
            .expect("Couldnt find root robot!"),
    );

    // Print out the characteristics
    root_peripheral.print_characteristics().await;

    // Subscribe to the BLE channel to start receiving messages
    root_peripheral.subscribe().await;

    let p_clone = root_peripheral.clone();
    // In a background thread enter a loop which reads any messages received from the device
    tokio::spawn(async move {
        p_clone.run_message_loop().await;
    });

    // Send a message to the device requesting the current versions
    root_peripheral.print_versions().await;

    // Turn on the lights
    root_peripheral
        .set_lights(LEDLightsState::Spin, 0x00, 0xFF, 0x00)
        .await;

    // Draw letter H
    // orchestrator::orchestrate(
    //     &root_peripheral,
    //     vec![
    //         vec![Point::new(10.0, 0.0), Point::new(70.0, 0.0)],
    //         vec![Point::new(40.0, 0.0), Point::new(40.0, 30.0)],
    //         vec![Point::new(10.0, 30.0), Point::new(70.0, 30.0)],
    //     ],
    // )
    // .await;

    // Draw letter y
    // orchestrator::orchestrate(
    //     &root_peripheral,
    //     vec![
    //         vec![Point::new(10.0, 10.0), Point::new(110.0, 110.0)],
    //         vec![Point::new(60.0, 60.0), Point::new(10.0, 110.0)],
    //     ],
    // )
    // .await;

    // Draw letter Q
    // orchestrator::orchestrate(
    //     &root_peripheral,
    //     vec![
    //         vec![Point::new(10.0, 10.0), Point::new(10.0, 50.0), Point::new(50.0, 50.0), Point::new(50.0, 10.0), Point::new(10.0, 10.0)],
    //         vec![Point::new(40.0, 20.0), Point::new(60.0, 0.0)]
    //     ],
    // )
    // .await;

    // Draw letter B
    // let mut orch = LinearOrchestrator::new();
    // orch.orchestrate(
    //     &root_peripheral,
    //     vec![
    //         vec![Point::new(10.0, 10.0), Point::new(10.0, 50.0)],
    //         vec![Point::new(25.0, 50.0)],
    //         vec![Point::new(35.0, 40.0)],
    //         vec![Point::new(25.0, 30.0)],
    //         vec![Point::new(10.0, 30.0)],
    //         vec![Point::new(25.0, 30.0)],
    //         vec![Point::new(35.0, 20.0)],
    //         vec![Point::new(25.0, 10.0)],
    //         vec![Point::new(10.0, 10.0)]
    //     ],
    // )
    // .await;

    // Draw letter B
    // let mut orch = LinearOrchestrator::new();
    // orch.orchestrate(
    //     &root_peripheral,
    //     vec![
    //         vec![Point::new(10.0, 10.0), Point::new(10.0, 50.0)],
    //         vec![
    //             Point::new(10.0, 50.0),
    //             Point::new(20.0, 40.0),
    //             Point::new(10.0, 30.0),
    //         ],
    //         vec![
    //             Point::new(10.0, 30.0),
    //             Point::new(20.0, 20.0),
    //             Point::new(10.0, 10.0),
    //         ],
    //     ],
    // )
    // .await;

    // Draw Heart
    let mut orch = LinearOrchestrator::new();
    orch.orchestrate(
        &root_peripheral,
        vec![
            vec![Point::new(0.0, -30.0), Point::new(-40.0, 30.0)],
            vec![
                Point::new(-40.0, 30.0),
                Point::new(-20.0, 50.0),
                Point::new(0.0, 30.0),
            ],
            vec![
                Point::new(0.0, 30.0),
                Point::new(20.0, 50.0),
                Point::new(40.0, 30.0),
            ],
            vec![Point::new(40.0, 30.0), Point::new(0.0, -30.0)],
        ],
    )
    .await;

    // root_peripheral.say_phrase("What").await;
    // root_peripheral.say_phrase("are").await;
    // root_peripheral.say_phrase("you").await;
    // root_peripheral.say_phrase("doing?").await;

    // Start with a small movement
    //root_peripheral.drive_distance(10).await;

    //root_peripheral.reset_position().await;

    //designs::draw_letter_H(&root_peripheral).await;

    // Disconnect
    root_peripheral.disconnect().await;

    Ok(())
}
