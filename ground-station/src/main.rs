// mod tasks;

// use eframe::{App, NativeOptions, egui}; // Import App and NativeOptions directly
// use std::{
//     io,
//     sync::mpsc, // Multi-producer, single-consumer channels
//     thread,
//     time::Duration,
// };

// use color_eyre::eyre;
// use serialport::{SerialPortType, UsbPortInfo};

// // --- Communication Enum ---
// // Messages sent FROM the serial thread TO the egui thread
// #[derive(Debug)]
// enum SerialMessage {
//     Data(Vec<u8>),     // Serial data read
//     Info(String),      // Status messages (connecting, waiting, closing)
//     Error(String),     // Error messages
//     Connected(String), // Confirmation with port name
//     Disconnected,      // Confirmation of disconnection or end of thread
// }

// // --- The function to run in the background thread ---
// // (Mostly unchanged from previous version)
// fn serial_reader_thread(
//     data_tx: mpsc::Sender<SerialMessage>,
//     stop_rx: mpsc::Receiver<()>,
// ) -> eyre::Result<()> {
//     let mut once = true;
//     let dongle_info = loop {
//         if stop_rx.try_recv().is_ok() {
//             // Use _ = to ignore potential send error if receiver is dropped
//             let _ = data_tx.send(SerialMessage::Info("Connection cancelled.".to_string()));
//             return Ok(());
//         }

//         match serialport::available_ports() {
//             Ok(ports) => {
//                 if let Some(dongle) = ports.into_iter().find(|info| match &info.port_type {
//                     // Match only the VID from UsbPortInfo
//                     SerialPortType::UsbPort(UsbPortInfo {
//                         vid: consts::USB_VID_DEMO,
//                         ..
//                     }) => true,
//                     _ => false,
//                 }) {
//                     break dongle;
//                 } else if once {
//                     once = false;
//                     let _ =
//                         data_tx.send(SerialMessage::Info("(Waiting for Dongle...)".to_string()));
//                 }
//             }
//             Err(e) => {
//                 let _ = data_tx.send(SerialMessage::Error(format!("List ports error: {}", e)));
//                 thread::sleep(Duration::from_secs(1));
//             }
//         }
//         thread::sleep(Duration::from_millis(500));
//     };

//     let _ = data_tx.send(SerialMessage::Info(format!(
//         "Found: {}",
//         dongle_info.port_name
//     )));

//     let mut port = match serialport::new(&dongle_info.port_name, 115_200)
//         .timeout(Duration::from_millis(20)) // Increased timeout slightly
//         .open()
//     {
//         Ok(p) => {
//             let _ = data_tx.send(SerialMessage::Connected(dongle_info.port_name.clone()));
//             p
//         }
//         Err(e) => {
//             let _ = data_tx.send(SerialMessage::Error(format!(
//                 "Failed to open {}: {}",
//                 dongle_info.port_name, e
//             )));
//             return Err(e.into());
//         }
//     };

//     let mut read_buf = [0u8; 1024];
//     loop {
//         if stop_rx.try_recv().is_ok() {
//             let _ = data_tx.send(SerialMessage::Info("Stop signal received.".to_string()));
//             break;
//         }

//         match port.read(&mut read_buf) {
//             Ok(0) => {
//                 // Sometimes 0 bytes read might indicate disconnection on some platforms
//                 // Let's treat it like a timeout for now, or add specific logic if needed
//                 // thread::sleep(Duration::from_millis(1)); // Optional small sleep
//             }
//             Ok(n) => {
//                 let _ = data_tx.send(SerialMessage::Data(read_buf[..n].to_vec()));
//             }
//             Err(e) if e.kind() == io::ErrorKind::TimedOut => {
//                 // Timeout is normal, continue to check stop signal
//             }
//             Err(e) => {
//                 let _ = data_tx.send(SerialMessage::Error(format!("Serial read error: {}", e)));
//                 break;
//             }
//         }
//     }

//     let _ = data_tx.send(SerialMessage::Info("(Closing port)".to_string()));
//     // Port closes automatically when `port` goes out of scope.
//     let _ = data_tx.send(SerialMessage::Disconnected); // Signal thread end

//     Ok(())
// }

// // --- Egui App ---
// struct MyApp {
//     terminal_output: String,
//     serial_thread_handle: Option<thread::JoinHandle<eyre::Result<()>>>,
//     serial_data_receiver: Option<mpsc::Receiver<SerialMessage>>,
//     serial_stop_sender: Option<mpsc::Sender<()>>,
//     // Simplified status tracking
//     is_connecting: bool, // Track if we are in the process of connecting
//     is_connected: bool,
//     status_message: String,
// }

// impl Default for MyApp {
//     fn default() -> Self {
//         Self {
//             terminal_output: "Press 'Connect' to start reading from serial port...\n".to_owned(),
//             serial_thread_handle: None,
//             serial_data_receiver: None,
//             serial_stop_sender: None,
//             is_connecting: false,
//             is_connected: false,
//             status_message: "Disconnected".to_string(),
//         }
//     }
// }

// impl App for MyApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         let mut request_repaint = false;
//         let mut reset_connection_state = false;
//         let mut received_connected_msg = None; // Store port name if connected this frame
//         let mut received_error_msg = None; // Store error if received this frame

//         // --- Process messages ---
//         // Check receiver without holding borrow across potential state change
//         if let Some(receiver) = &self.serial_data_receiver {
//             for msg in receiver.try_iter() {
//                 match msg {
//                     SerialMessage::Data(bytes) => {
//                         self.terminal_output
//                             .push_str(&String::from_utf8_lossy(&bytes));
//                         request_repaint = true; // Ensure UI updates if lots of data
//                     }
//                     SerialMessage::Info(info) => {
//                         self.terminal_output.push_str(&format!("[INFO] {}\n", info));
//                         // Don't overwrite crucial status like "Connected" with transient info
//                         if !self.is_connected && !self.is_connecting {
//                             self.status_message = info;
//                         }
//                     }
//                     SerialMessage::Error(err) => {
//                         let err_msg = format!("[ERROR] {}\n", err);
//                         self.terminal_output.push_str(&err_msg);
//                         received_error_msg = Some(err); // Store error to update status later
//                         reset_connection_state = true; // Signal to reset connection state
//                     }
//                     SerialMessage::Connected(port_name) => {
//                         self.terminal_output
//                             .push_str(&format!("[INFO] Connected to {}\n", port_name));
//                         received_connected_msg = Some(port_name); // Mark connected
//                         // Don't set is_connected yet, do it after the loop
//                     }
//                     SerialMessage::Disconnected => {
//                         self.terminal_output
//                             .push_str("[INFO] Disconnected / Thread ended.\n");
//                         reset_connection_state = true; // Signal to reset connection state
//                     }
//                 }
//             }
//         }

//         // --- Handle state changes AFTER processing messages ---
//         if let Some(port_name) = received_connected_msg {
//             self.is_connected = true;
//             self.is_connecting = false;
//             self.status_message = format!("Connected to {}", port_name);
//         }

//         if let Some(error_text) = received_error_msg {
//             self.status_message = format!("Error: {}", error_text);
//             // reset_connection_state is already true
//         }

//         if reset_connection_state {
//             // This is where the cleanup happens, now safely outside the receiver borrow scope
//             self.serial_thread_handle = None; // Drop the handle
//             self.serial_data_receiver = None;
//             self.serial_stop_sender = None;
//             self.is_connected = false;
//             self.is_connecting = false;
//             // Keep error status message if it was set, otherwise reset to Disconnected
//             if !self.status_message.starts_with("Error") {
//                 self.status_message = "Disconnected".to_string();
//             }
//         }

//         // --- Check if thread panicked or finished unexpectedly ---
//         // Use take() to check and remove handle if finished, avoiding borrow issues
//         let mut thread_finished_unexpectedly = false;
//         if let Some(handle) = &self.serial_thread_handle {
//             if handle.is_finished() {
//                 if let Some(handle) = self.serial_thread_handle.take() {
//                     match handle.join() {
//                         Ok(Ok(())) => { /* Normal finish, likely got Disconnected msg */ }
//                         Ok(Err(e)) => {
//                             // Thread returned error
//                             self.terminal_output
//                                 .push_str(&format!("[THREAD ERR] {}\n", e));
//                             self.status_message = format!("Thread Error: {}", e);
//                         }
//                         Err(panic_payload) => {
//                             // Thread panicked
//                             self.terminal_output
//                                 .push_str(&format!("[PANIC] {:?}\n", panic_payload));
//                             self.status_message = "Thread Panic!".to_string();
//                         }
//                     }
//                     thread_finished_unexpectedly = true;
//                 }
//             }
//         }
//         // If thread finished without explicit disconnect signal, reset state
//         if thread_finished_unexpectedly && !reset_connection_state {
//             self.serial_data_receiver = None;
//             self.serial_stop_sender = None;
//             self.is_connected = false;
//             self.is_connecting = false;
//             if !self.status_message.starts_with("Error")
//                 && !self.status_message.starts_with("Thread")
//             {
//                 self.status_message = "Disconnected (Thread)".to_string();
//             }
//         }

//         // --- UI Definition ---
//         egui::TopBottomPanel::top("controls_panel").show(ctx, |ui| {
//             ui.horizontal(|ui| {
//                 ui.label("Status:");
//                 ui.label(&self.status_message); // Show current status

//                 ui.separator();

//                 // Connect Button
//                 let can_connect = !self.is_connected
//                     && !self.is_connecting
//                     && self.serial_thread_handle.is_none();
//                 if ui
//                     .add_enabled(can_connect, egui::Button::new("Connect"))
//                     .clicked()
//                 {
//                     self.terminal_output
//                         .push_str("[CMD] Attempting to connect...\n");
//                     self.status_message = "Connecting...".to_string();
//                     self.is_connecting = true;
//                     self.is_connected = false; // Ensure not connected while connecting

//                     let (data_tx, data_rx) = mpsc::channel();
//                     let (stop_tx, stop_rx) = mpsc::channel();

//                     self.serial_data_receiver = Some(data_rx);
//                     self.serial_stop_sender = Some(stop_tx);

//                     let handle = thread::spawn(move || serial_reader_thread(data_tx, stop_rx));
//                     self.serial_thread_handle = Some(handle);
//                 }

//                 // Disconnect Button
//                 let can_disconnect = self.is_connected || self.is_connecting;
//                 if ui
//                     .add_enabled(can_disconnect, egui::Button::new("Disconnect"))
//                     .clicked()
//                 {
//                     if let Some(sender) = self.serial_stop_sender.as_ref() {
//                         self.terminal_output
//                             .push_str("[CMD] Sending stop signal...\n");
//                         self.status_message = "Disconnecting...".to_string();
//                         let _ = sender.send(()); // Ignore error if thread already exited
//                         self.is_connecting = false; // Stop saying "connecting" if we were
//                     } else {
//                         // If no sender, maybe thread died? Reset state just in case.
//                         self.terminal_output
//                             .push_str("[WARN] No active connection to disconnect.\n");
//                         reset_connection_state = true; // Reuse flag to trigger cleanup below
//                         self.status_message = "Disconnected".to_string();
//                     }
//                 }

//                 ui.separator();

//                 if ui.button("Clear Output").clicked() {
//                     self.terminal_output.clear();
//                 }
//             });
//         });

//         // Re-check reset flag in case disconnect button found no sender
//         if reset_connection_state {
//             self.serial_thread_handle = None;
//             self.serial_data_receiver = None;
//             self.serial_stop_sender = None;
//             self.is_connected = false;
//             self.is_connecting = false;
//             if !self.status_message.starts_with("Error") {
//                 self.status_message = "Disconnected".to_string();
//             }
//         }

//         egui::CentralPanel::default().show(ctx, |ui| {
//             // Reserve space, but the bottom panel will draw over it
//             ui.heading("Serial Output");
//             ui.separator();
//         });

//         egui::TopBottomPanel::bottom("terminal_panel")
//             .resizable(true)
//             .min_height(100.0)
//             .show(ctx, |ui| {
//                 egui::ScrollArea::vertical()
//                     .stick_to_bottom(true)
//                     .auto_shrink([false, false]) // Prevent shrinkage
//                     .show(ui, |ui| {
//                         let mut output_display = self.terminal_output.as_str();
//                         ui.add_sized(
//                             ui.available_size(), // Fill available space
//                             egui::TextEdit::multiline(&mut output_display)
//                                 .font(egui::TextStyle::Monospace)
//                                 .desired_width(f32::INFINITY)
//                                 .lock_focus(false), // Allow focus/selection
//                         );
//                     });
//             });

//         // Request repaint if data was received
//         if request_repaint {
//             ctx.request_repaint();
//         }
//     }
// }

// fn main() -> Result<(), eframe::Error> {
//     // color_eyre::install().expect("Failed to install color_eyre"); // Optional

//     let options = NativeOptions {
//         viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
//         ..Default::default()
//     };
//     // Corrected closure signature
//     eframe::run_native(
//         "Ground station",
//         options,
//         Box::new(|_cc| Ok(Box::<MyApp>::default())), // Needs Ok() wrapper
//     )
// }

use eframe::{egui, App, NativeOptions};
use serialport::{SerialPortType, UsbPortInfo};
use std::{
    io,
    sync::mpsc, // Multi-producer, single-consumer channels
    thread,
    time::Duration,
};
// Using color_eyre for better error reports if needed, but not strictly required by the simplification
use color_eyre::eyre;

// Placeholder: Define the USB Vendor ID you are looking for.
// Replace this with the actual VID from your consts module or configuration.
const USB_VID_DEMO: u16 = 0x1234; // Example VID

// --- Communication Enum ---
// Messages sent FROM the serial thread TO the egui thread (Unchanged)
#[derive(Debug)]
enum SerialMessage {
    Data(Vec<u8>),
    Info(String),
    Error(String),
    Connected(String), // Port name
    Disconnected,
}

// --- Background Thread Function ---
// (Mostly unchanged, slight simplification in initial message)
fn serial_reader_thread(
    data_tx: mpsc::Sender<SerialMessage>,
    stop_rx: mpsc::Receiver<()>,
) -> eyre::Result<()> {
    // --- Find Port ---
    let dongle_info = loop {
        // Check for stop signal before searching
        if stop_rx.try_recv().is_ok() {
            let _ = data_tx.send(SerialMessage::Info("Connection cancelled.".to_string()));
            // Signal thread end immediately if cancelled before connection
            let _ = data_tx.send(SerialMessage::Disconnected);
            return Ok(());
        }

        match serialport::available_ports() {
            Ok(ports) => {
                if let Some(dongle) = ports.into_iter().find(|info| {
                    matches!(
                        &info.port_type,
                        SerialPortType::UsbPort(UsbPortInfo { vid: USB_VID_DEMO, .. })
                    )
                }) {
                    let _ = data_tx.send(SerialMessage::Info(format!(
                        "Found: {}",
                        dongle.port_name
                    )));
                    break dongle; // Found the port, exit loop
                } else {
                    // Send waiting message only if no port is found yet in this iteration
                    let _ =
                        data_tx.send(SerialMessage::Info("(Waiting for Dongle...)".to_string()));
                }
            }
            Err(e) => {
                // Send error but keep trying (unless stop signal received)
                let _ = data_tx.send(SerialMessage::Error(format!("List ports error: {}", e)));
                thread::sleep(Duration::from_secs(1)); // Wait before retrying list ports
            }
        }
        // Wait before next check if port not found
        thread::sleep(Duration::from_millis(500));
    };

    // --- Open Port ---
    let mut port = match serialport::new(&dongle_info.port_name, 115_200)
        .timeout(Duration::from_millis(20))
        .open()
    {
        Ok(p) => {
            // Send Connected *after* successfully opening
            let _ = data_tx.send(SerialMessage::Connected(dongle_info.port_name.clone()));
            p
        }
        Err(e) => {
            let err_msg = format!("Failed to open {}: {}", dongle_info.port_name, e);
            let _ = data_tx.send(SerialMessage::Error(err_msg.clone()));
            let _ = data_tx.send(SerialMessage::Disconnected); // Signal thread end on error
            return Err(eyre::eyre!(err_msg)); // Return specific error
        }
    };

    // --- Read Loop ---
    let mut read_buf = [0u8; 1024];
    loop {
        // Check for stop signal first
        if stop_rx.try_recv().is_ok() {
            let _ = data_tx.send(SerialMessage::Info("Stop signal received.".to_string()));
            break; // Exit loop to close port and end thread
        }

        match port.read(&mut read_buf) {
            Ok(0) => {
                // Potentially indicates disconnection on some platforms, treat like timeout
                // thread::sleep(Duration::from_millis(1)); // Optional small delay
            }
            Ok(n) => {
                // Send received data
                let _ = data_tx.send(SerialMessage::Data(read_buf[..n].to_vec()));
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                // Timeout is expected, continue loop to check stop signal again
            }
            Err(e) => {
                // Report serial read errors and exit loop
                let _ = data_tx.send(SerialMessage::Error(format!("Serial read error: {}", e)));
                break;
            }
        }
    }

    // --- Cleanup ---
    let _ = data_tx.send(SerialMessage::Info("(Closing port)".to_string()));
    // Port is closed automatically when `port` goes out of scope here.
    let _ = data_tx.send(SerialMessage::Disconnected); // Signal that the thread is done

    Ok(())
}

// --- Simplified Application State ---
#[derive(Debug)]
enum AppState {
    Disconnected,
    Connecting,
    Connected(String), // Holds the port name
    Error(String),     // Holds the error message
}

// --- Structure to hold active connection resources ---
struct ActiveConnection {
    #[allow(dead_code)] // handle is currently only used for is_finished/join
    handle: thread::JoinHandle<eyre::Result<()>>,
    data_rx: mpsc::Receiver<SerialMessage>,
    stop_tx: mpsc::Sender<()>,
}

// --- Egui App ---
struct MyApp {
    terminal_output: String,
    state: AppState,
    // Connection resources are now optional and bundled
    connection: Option<ActiveConnection>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            terminal_output: "Press 'Connect' to start.\n".to_owned(),
            state: AppState::Disconnected,
            connection: None,
        }
    }
}

impl MyApp {
    /// Helper function to clean up connection resources and set state.
    /// Optionally preserves an Error state.
    fn disconnect_internal(&mut self, became_error: Option<String>) {
        self.connection = None; // Drop connection resources (closes channels, allows join)
        if let Some(err) = became_error {
            // If an error was the cause, set state to Error
            self.state = AppState::Error(err);
        } else {
            // Otherwise, only transition to Disconnected if not already in an Error state
            if !matches!(self.state, AppState::Error(_)) {
                self.state = AppState::Disconnected;
            }
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut request_repaint = false;
        let mut disconnect_reason: Option<Option<String>> = None; // None = no disconnect, Some(None) = clean disconnect, Some(Some(err)) = error disconnect

        // --- Process messages from Serial Thread ---
        if let Some(conn) = &self.connection {
            for msg in conn.data_rx.try_iter() {
                match msg {
                    SerialMessage::Data(bytes) => {
                        self.terminal_output
                            .push_str(&String::from_utf8_lossy(&bytes));
                        request_repaint = true; // Repaint if we got data
                    }
                    SerialMessage::Info(info) => {
                        self.terminal_output.push_str(&format!("[INFO] {}\n", info));
                        // Update state only if we are still in the 'Connecting' phase
                        if matches!(self.state, AppState::Connecting) {
                           // Maybe update status temporarily, but avoid overwriting final state
                           // self.state = AppState::Connecting; // Redundant? Or show sub-status?
                        }
                    }
                    SerialMessage::Error(err) => {
                        let err_msg = format!("[ERROR] {}\n", err);
                        self.terminal_output.push_str(&err_msg);
                        // Signal to disconnect with an error state
                        disconnect_reason = Some(Some(err));
                    }
                    SerialMessage::Connected(port_name) => {
                        self.terminal_output
                            .push_str(&format!("[INFO] Connected to {}\n", port_name));
                        // Transition to Connected state
                        self.state = AppState::Connected(port_name);
                    }
                    SerialMessage::Disconnected => {
                        self.terminal_output
                            .push_str("[INFO] Disconnected / Thread ended.\n");
                        // Signal a clean disconnect
                        disconnect_reason = Some(None);
                    }
                }
            }
        }

        // --- Handle Disconnection Request (from messages) ---
        if let Some(reason) = disconnect_reason {
            self.disconnect_internal(reason);
        }

        // --- Check if Thread Panicked or Finished Unexpectedly ---
        let mut thread_finished_unexpectedly = false;
        if let Some(conn) = &self.connection {
            if conn.handle.is_finished() {
                thread_finished_unexpectedly = true;
            }
        }
        // Handle join outside the borrow
        if thread_finished_unexpectedly {
             if let Some(conn) = self.connection.take() { // Take ownership to join
                let mut error_message = "Thread finished unexpectedly.".to_string();
                match conn.handle.join() {
                    Ok(Ok(())) => { /* Normal finish, likely disconnect msg already handled */
                       self.terminal_output.push_str("[INFO] Serial thread joined cleanly.\n");
                       // If disconnect wasn't signaled yet, do it now
                       if matches!(self.state, AppState::Connected(_) | AppState::Connecting) {
                           disconnect_reason = Some(None);
                       }
                    }
                    Ok(Err(e)) => { // Thread returned an eyre::Error
                        error_message = format!("Thread Error: {}", e);
                        self.terminal_output.push_str(&format!("[THREAD ERR] {}\n", e));
                        disconnect_reason = Some(Some(error_message.clone()));
                    }
                    Err(panic_payload) => { // Thread panicked
                        error_message = format!("Thread Panic: {:?}", panic_payload);
                        self.terminal_output.push_str(&format!("[PANIC] {:?}\n", panic_payload));
                        disconnect_reason = Some(Some(error_message.clone()));
                    }
                }
                 // Ensure state reflects the error/disconnection if not already handled
                if let Some(reason) = disconnect_reason {
                    self.disconnect_internal(reason);
                } else if !matches!(self.state, AppState::Disconnected | AppState::Error(_)) {
                    // If thread ended but no specific message caused disconnect, force it
                    self.disconnect_internal(None);
                }
            }
        }


        // --- UI Definition ---
        egui::TopBottomPanel::top("controls_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Display Status based on state
                ui.label("Status:");
                let status_text = match &self.state {
                    AppState::Disconnected => "Disconnected".to_string(),
                    AppState::Connecting => "Connecting...".to_string(),
                    AppState::Connected(port) => format!("Connected to {}", port),
                    AppState::Error(err) => format!("Error: {}", err),
                };
                ui.label(status_text);

                ui.separator();

                // Connect Button - Enabled only when Disconnected or Error
                let can_connect = matches!(self.state, AppState::Disconnected | AppState::Error(_));
                if ui.add_enabled(can_connect, egui::Button::new("Connect")).clicked() {
                    self.terminal_output.push_str("[CMD] Attempting to connect...\n");
                    self.state = AppState::Connecting; // Set state immediately

                    let (data_tx, data_rx) = mpsc::channel();
                    let (stop_tx, stop_rx) = mpsc::channel();

                    // Spawn thread
                    let handle = thread::spawn(move || serial_reader_thread(data_tx, stop_rx));

                    // Store connection resources
                    self.connection = Some(ActiveConnection {
                        handle,
                        data_rx,
                        stop_tx,
                    });
                }

                // Disconnect Button - Enabled only when Connecting or Connected
                let can_disconnect = matches!(self.state, AppState::Connecting | AppState::Connected(_));
                if ui.add_enabled(can_disconnect, egui::Button::new("Disconnect")).clicked() {
                    if let Some(conn) = &self.connection {
                        self.terminal_output.push_str("[CMD] Sending stop signal...\n");
                        // Attempt to send stop signal, ignore error if thread already closed
                        let _ = conn.stop_tx.send(());
                        // Optionally change state to "Disconnecting" or directly to Disconnected
                        // self.state = AppState::Disconnected; // Let Disconnected message handle final state
                    } else {
                        // Should not happen if button is enabled correctly, but handle defensively
                        self.terminal_output.push_str("[WARN] No active connection to disconnect.\n");
                        self.disconnect_internal(None); // Ensure state is clean
                    }
                }

                ui.separator();

                if ui.button("Clear Output").clicked() {
                    self.terminal_output.clear();
                }
            });
        });

        // Central panel for spacing, content is in top/bottom panels
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add some placeholder content or leave it empty if top/bottom panels cover everything.
            ui.vertical_centered(|ui| {
                 ui.heading("Serial Output");
            });
        });


        // Bottom panel for the terminal output
        egui::TopBottomPanel::bottom("terminal_panel")
            .resizable(true)
            .min_height(100.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false]) // Prevent shrinkage
                    .show(ui, |ui| {
                        // Use TextEdit for viewing, but make it immutable borrow
                        let mut output_display = self.terminal_output.as_str();
                        ui.add_sized(
                            ui.available_size(), // Fill available space
                            egui::TextEdit::multiline(&mut output_display)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .lock_focus(true), // Lock focus to prevent accidental edits if desired
                        );
                    });
            });

        // Request repaint if data was received or state potentially changed implicitly
        if request_repaint || thread_finished_unexpectedly {
            ctx.request_repaint();
        }
    }
}

// --- Main Function --- (Unchanged, added Ok wrapper)
fn main() -> Result<(), eframe::Error> {
    // Optional: install color_eyre for better panic messages
    // color_eyre::install().expect("Failed to install color_eyre");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Serial Monitor", // Changed title slightly
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())), // Wrap the Box in Ok()
    )
}