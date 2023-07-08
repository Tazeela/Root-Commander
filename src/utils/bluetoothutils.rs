use crate::irobot::root::{is_root_robot, RootRobot};

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;

// Helper which uses the btleplug library to scan available peripherals looking for a specific device.
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
