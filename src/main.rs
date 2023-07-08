use std::{error::Error, sync::Arc};

mod utils;
mod orchestrator;

mod irobot;
use irobot::root::{LEDLightsState, RootRobot};

use orchestrator::{LinearOrchestrator};
use utils::{find_root_peripheral, Point};

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

    root_peripheral.say_phrase("What").await;
    root_peripheral.say_phrase("are").await;
    root_peripheral.say_phrase("you").await;
    root_peripheral.say_phrase("doing?").await;

    // Start with a small movement
    //root_peripheral.drive_distance(10).await;

    //root_peripheral.reset_position().await;

    //designs::draw_letter_H(&root_peripheral).await;

    // Disconnect
    root_peripheral.disconnect().await;

    Ok(())
}
