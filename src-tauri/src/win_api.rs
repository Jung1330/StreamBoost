//! Safe wrappers around standard Windows Process, Power, and Multimedia Timer APIs.
//!
//! Security guarantee:
//! - Only uses standard process information/priority/affinity/EcoQoS APIs.
//! - NEVER uses memory read/write/injection APIs.
//! - NEVER uses REALTIME_PRIORITY_CLASS (max is HIGH_PRIORITY_CLASS).

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    GetPriorityClass, GetProcessAffinityMask, OpenProcess, ProcessPowerThrottling,
    SetPriorityClass, SetProcessAffinityMask, SetProcessInformation, HIGH_PRIORITY_CLASS,
    PROCESS_CREATION_FLAGS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Standard Windows High Performance Power Plan GUID.
pub const HIGH_PERFORMANCE_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
/// Standard Windows Ultimate Performance Power Plan GUID (fallback).
pub const ULTIMATE_PERFORMANCE_GUID: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

/// Returns the number of logical CPU cores on the system.
pub fn get_logical_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

#[cfg(windows)]
struct OwnedHandle(HANDLE);

#[cfg(windows)]
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Applies `HIGH_PRIORITY_CLASS` to a process and returns its previous priority class.
/// Never applies `REALTIME_PRIORITY_CLASS`.
#[cfg(windows)]
pub fn set_process_high_priority(pid: u32) -> Result<u32, String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        )
        .map_err(|e| format!("OpenProcess({pid}) failed: {e}"))?;
        let guard = OwnedHandle(handle);

        let previous = GetPriorityClass(guard.0);
        if previous == 0 {
            return Err(format!("GetPriorityClass({pid}) returned 0"));
        }

        // Only elevate to HIGH_PRIORITY_CLASS (never REALTIME_PRIORITY_CLASS).
        if previous != HIGH_PRIORITY_CLASS.0 {
            SetPriorityClass(guard.0, HIGH_PRIORITY_CLASS)
                .map_err(|e| format!("SetPriorityClass({pid}) failed: {e}"))?;
        }

        Ok(previous)
    }
}

/// Restores a process's priority class to a previously saved value.
#[cfg(windows)]
pub fn restore_process_priority(pid: u32, original_priority: u32) -> Result<(), String> {
    if original_priority == 0 {
        return Ok(());
    }
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess({pid}) failed: {e}"))?;
        let guard = OwnedHandle(handle);

        SetPriorityClass(guard.0, PROCESS_CREATION_FLAGS(original_priority))
            .map_err(|e| format!("SetPriorityClass({pid}) restore failed: {e}"))?;
        Ok(())
    }
}

/// Applies a CPU affinity mask to a process and returns its previous affinity mask.
#[cfg(windows)]
pub fn set_process_affinity(pid: u32, desired_cores: &[usize]) -> Result<usize, String> {
    if desired_cores.is_empty() {
        return Err("Desired cores list is empty".into());
    }

    let mut desired_mask: usize = 0;
    for &core_idx in desired_cores {
        if core_idx < usize::BITS as usize {
            desired_mask |= 1usize << core_idx;
        }
    }
    if desired_mask == 0 {
        return Err("Computed affinity mask is 0".into());
    }

    unsafe {
        let handle = OpenProcess(
            PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
            false,
            pid,
        )
        .map_err(|e| format!("OpenProcess({pid}) affinity failed: {e}"))?;
        let guard = OwnedHandle(handle);

        let mut process_mask: usize = 0;
        let mut system_mask: usize = 0;
        GetProcessAffinityMask(guard.0, &mut process_mask, &mut system_mask)
            .map_err(|e| format!("GetProcessAffinityMask({pid}) failed: {e}"))?;

        let safe_mask = desired_mask & system_mask;
        if safe_mask == 0 {
            return Err("Requested cores are outside the system affinity mask".into());
        }

        if safe_mask != process_mask {
            SetProcessAffinityMask(guard.0, safe_mask)
                .map_err(|e| format!("SetProcessAffinityMask({pid}) failed: {e}"))?;
        }

        Ok(process_mask)
    }
}

/// Restores a process's CPU affinity mask to its previously saved value.
#[cfg(windows)]
pub fn restore_process_affinity(pid: u32, original_mask: usize) -> Result<(), String> {
    if original_mask == 0 {
        return Ok(());
    }
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess({pid}) affinity restore failed: {e}"))?;
        let guard = OwnedHandle(handle);

        SetProcessAffinityMask(guard.0, original_mask)
            .map_err(|e| format!("SetProcessAffinityMask({pid}) restore failed: {e}"))?;
        Ok(())
    }
}

/// Enables or disables Windows Efficiency Mode (EcoQoS) on a process using
/// the official `SetProcessInformation(ProcessPowerThrottling)` Win32 API.
#[cfg(windows)]
pub fn set_process_eco_qos(pid: u32, enable: bool) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess({pid}) EcoQoS failed: {e}"))?;
        let guard = OwnedHandle(handle);

        let state = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: if enable {
                PROCESS_POWER_THROTTLING_EXECUTION_SPEED
            } else {
                0
            },
        };

        SetProcessInformation(
            guard.0,
            ProcessPowerThrottling,
            &state as *const PROCESS_POWER_THROTTLING_STATE as *const std::ffi::c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
        .map_err(|e| format!("SetProcessInformation({pid}, EcoQoS={enable}) failed: {e}"))?;

        Ok(())
    }
}

/// Enables 1ms high-resolution multimedia system timer (`timeBeginPeriod(1)`).
#[cfg(windows)]
pub fn enable_high_res_timer() -> bool {
    unsafe { timeBeginPeriod(1) == 0 }
}

/// Disables 1ms high-resolution multimedia system timer (`timeEndPeriod(1)`).
#[cfg(windows)]
pub fn disable_high_res_timer() -> bool {
    unsafe { timeEndPeriod(1) == 0 }
}

/// Reads the currently active Windows power scheme GUID via `powercfg /getactivescheme`.
/// Works regardless of Windows display language by extracting the 36-char GUID.
pub fn get_active_power_scheme() -> Result<String, String> {
    let mut cmd = Command::new("powercfg");
    cmd.arg("/getactivescheme");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run powercfg: {e}"))?;

    if !output.status.success() {
        return Err("powercfg /getactivescheme returned non-zero exit code".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_guid(&stdout).ok_or_else(|| format!("Could not parse GUID from powercfg: {stdout}"))
}

/// Switches Windows power scheme to the specified GUID.
pub fn set_power_scheme(guid: &str) -> Result<(), String> {
    let mut cmd = Command::new("powercfg");
    cmd.args(["/setactive", guid]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run powercfg /setactive: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("powercfg /setactive {guid} failed: {stderr}"))
    }
}

/// Attempts to switch to High Performance (or Ultimate Performance fallback).
pub fn set_high_performance_power_scheme() -> Result<String, String> {
    if set_power_scheme(HIGH_PERFORMANCE_GUID).is_ok() {
        return Ok(HIGH_PERFORMANCE_GUID.to_string());
    }
    if set_power_scheme(ULTIMATE_PERFORMANCE_GUID).is_ok() {
        return Ok(ULTIMATE_PERFORMANCE_GUID.to_string());
    }
    Err("High Performance power scheme is not available on this system".into())
}

fn extract_guid(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != '-');
        if cleaned.len() == 36 {
            let bytes = cleaned.as_bytes();
            if bytes[8] == b'-'
                && bytes[13] == b'-'
                && bytes[18] == b'-'
                && bytes[23] == b'-'
                && cleaned
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == '-')
            {
                return Some(cleaned.to_ascii_lowercase());
            }
        }
    }
    None
}

// Non-Windows stubs for cross-platform check safety
#[cfg(not(windows))]
pub fn set_process_high_priority(_pid: u32) -> Result<u32, String> {
    Ok(0)
}
#[cfg(not(windows))]
pub fn restore_process_priority(_pid: u32, _original_priority: u32) -> Result<(), String> {
    Ok(())
}
#[cfg(not(windows))]
pub fn set_process_affinity(_pid: u32, _desired_cores: &[usize]) -> Result<usize, String> {
    Ok(0)
}
#[cfg(not(windows))]
pub fn restore_process_affinity(_pid: u32, _original_mask: usize) -> Result<(), String> {
    Ok(())
}
#[cfg(not(windows))]
pub fn set_process_eco_qos(_pid: u32, _enable: bool) -> Result<(), String> {
    Ok(())
}
#[cfg(not(windows))]
pub fn enable_high_res_timer() -> bool {
    false
}
#[cfg(not(windows))]
pub fn disable_high_res_timer() -> bool {
    false
}
