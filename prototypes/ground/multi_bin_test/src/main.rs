use std::env;
use std::process::{Command, exit};
use std::path::PathBuf;
#[cfg(unix)] // Only compile the exec part on Unix-like systems
use exec::Command as ExecCommand;


fn main() {
    // --- Configuration ---
    let script_name = "launch_tmux.sh";
    let tmux_session_name = "multi_rust_apps";
    // --- End Configuration ---

    println!("Launcher started...");

    // 1. Find the launch script relative to the executable.
    // Assumes the script is in the project root, and the executable is in target/debug/ or target/release/
    let current_exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("Error getting current executable path: {}", e);
            exit(1);
        }
    };

    // target/debug/launcher -> target/debug -> target -> project_root
    let target_dir = current_exe.parent().unwrap_or_else(|| {
        eprintln!("Error: Cannot get parent directory of executable.");
        exit(1);
    });
    let project_root = target_dir.parent().unwrap_or_else(|| {
         eprintln!("Error: Cannot get project root directory from executable path.");
         exit(1);
    }).parent().unwrap_or_else(|| {
         eprintln!("Error: Cannot get project root directory from executable path.");
         exit(1);
    });


    let script_path = project_root.join(script_name);

    if !script_path.exists() {
        eprintln!("Error: Launch script not found at expected location: {}", script_path.display());
        eprintln!("Make sure '{}' is in the project root directory.", script_name);
        exit(1);
    }

    println!("Found launch script: {}", script_path.display());

    // 2. Execute the launch script using bash.
    println!("Executing launch script...");
    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);
    // Run script from project root context if needed (usually not necessary if script handles paths well)
    // cmd.current_dir(project_root);

    let script_status = match cmd.status() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("Error executing launch script '{}': {}", script_path.display(), e);
            eprintln!("Ensure 'bash' is installed and in your PATH.");
            exit(1);
        }
    };

    if !script_status.success() {
        eprintln!("Launch script failed with status: {}", script_status);
        exit(1);
    }

    println!("Launch script completed successfully.");

    // 3. Attach to the tmux session using exec (Unix-like systems only).
    // The 'exec' call replaces the current process (the launcher) with tmux.
    #[cfg(unix)]
    {
        println!("Attaching to tmux session '{}'...", tmux_session_name);
        let err = ExecCommand::new("tmux")
            .args(&["attach-session", "-t", tmux_session_name])
            .exec();
        // If exec returns, it's an error
        eprintln!("Error trying to exec into tmux: {}", err);
        exit(1);
    }

    // If not on Unix, exec is not available. Print a message.
    #[cfg(not(unix))]
    {
         println!("-----------------------------------------------------");
         println!("Automatic attachment via 'exec' is only supported on Unix-like systems (Linux, macOS).");
         println!("Please attach manually using:");
         println!("  tmux attach-session -t {}", tmux_session_name);
         println!("-----------------------------------------------------");
         exit(0); // Exit successfully, as the script ran.
    }
}
