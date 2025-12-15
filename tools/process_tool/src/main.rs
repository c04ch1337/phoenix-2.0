
use serde::Serialize;
use std::env;
use sysinfo::{Pid, System};

#[derive(Serialize)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_usage: u64, // in KB
}

/// Lists all running processes.
pub fn list_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes = Vec::new();
    for (pid, process) in sys.processes() {
        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
        });
    }
    processes
}

/// Terminates a process by its PID.
///
/// # Arguments
///
/// * `pid` - The Process ID of the process to terminate.
///
/// # Returns
///
/// * `Ok(())` if the process was terminated successfully.
/// * `Err(String)` if the process could not be terminated.
pub fn kill_process(pid: u32) -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let pid = Pid::from_u32(pid);
    if let Some(process) = sys.process(pid) {
        if process.kill() {
            Ok(())
        } else {
            Err(format!("Failed to kill process with PID {}", pid.as_u32()))
        }
    } else {
        Err(format!("Process with PID {} not found", pid.as_u32()))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: process_tool <list|kill> [pid]");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "list" => {
            let processes = list_processes();
            println!("{}", serde_json::to_string_pretty(&processes).unwrap());
        }
        "kill" => {
            if args.len() < 3 {
                println!("Usage: process_tool kill <pid>");
                return;
            }
            let pid_str = &args[2];
            match pid_str.parse::<u32>() {
                Ok(pid) => {
                    match kill_process(pid) {
                        Ok(()) => println!("Process with PID {} killed successfully.", pid),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Err(_) => {
                    println!("Invalid PID: {}", pid_str);
                }
            }
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}
