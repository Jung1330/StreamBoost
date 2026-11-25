/// Critical Windows system processes, security services, anti-cheat services,
/// and StreamBoost itself. These processes must NEVER be modified (priority,
/// affinity, or EcoQoS).
const PROTECTED_PROCESSES: &[&str] = &[
    // Windows Kernel & Core System
    "system",
    "system idle process",
    "idle",
    "registry",
    "memory compression",
    "secure system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "services.exe",
    "lsass.exe",
    "lsaiso.exe",
    "svchost.exe",
    "fontdrvhost.exe",
    "winlogon.exe",
    "dwm.exe",
    "explorer.exe",
    "sihost.exe",
    "taskhostw.exe",
    "ctfmon.exe",
    "runtimebroker.exe",
    "searchindexer.exe",
    "spoolsv.exe",
    "audiodg.exe",
    "conhost.exe",
    "dashost.exe",
    "dllhost.exe",
    "wudfhost.exe",
    "wmiprvse.exe",
    "trustedinstaller.exe",
    "tiworker.exe",
    "ntoskrnl.exe",
    "logonui.exe",
    "shellexperiencehost.exe",
    "startmenuexperiencehost.exe",
    "textinputhost.exe",
    // Windows Defender & Security
    "msmpeng.exe",
    "nissrv.exe",
    "securityhealthservice.exe",
    "securityhealthsystray.exe",
    "smartscreen.exe",
    // Anti-Cheat Services (never touch)
    "vgc.exe",
    "vgtray.exe",
    "easyanticheat.exe",
    "easyanticheat_eos.exe",
    "beservice.exe",
    "bedaisy.exe",
    "faceitservice.exe",
    "faceitclient.exe",
    "punkbustera.exe",
    "punkbusterb.exe",
    "nprotect.exe",
    // StreamBoost itself
    "streamboost.exe",
];

/// Returns `true` if the given process executable name is protected and must
/// never be modified by StreamBoost.
pub fn is_protected_process(exe_name: &str) -> bool {
    let normalized = exe_name
        .trim()
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(exe_name)
        .to_ascii_lowercase();

    if normalized.is_empty() {
        return true;
    }

    PROTECTED_PROCESSES.contains(&normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_processes_are_protected() {
        assert!(is_protected_process("csrss.exe"));
        assert!(is_protected_process("LSASS.EXE"));
        assert!(is_protected_process("C:\\Windows\\System32\\dwm.exe"));
        assert!(is_protected_process("svchost.exe"));
        assert!(is_protected_process("explorer.exe"));
        assert!(is_protected_process("System"));
        assert!(is_protected_process("vgc.exe"));
        assert!(is_protected_process("StreamBoost.exe"));
    }

    #[test]
    fn normal_apps_are_allowed() {
        assert!(!is_protected_process("chrome.exe"));
        assert!(!is_protected_process("Discord.exe"));
        assert!(!is_protected_process("cs2.exe"));
    }
}
