use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum MacFramework {
    SeLinux(SeLinuxInfo),
    AppArmor(AppArmorInfo),
    None,
}

#[derive(Debug)]
pub struct SeLinuxInfo {
    pub enforcing: bool,
    pub context:   String,
    pub confined:  bool,  // false = unconfined_t, true = some other domain
}

#[derive(Debug)]
pub struct AppArmorInfo {
    pub profile:   String,
    pub confined:  bool,  // false = "unconfined"
}

impl MacFramework {
    /// Detect which MAC framework is active, if any.
    pub fn detect() -> Self {
        // SELinux mounts at /sys/fs/selinux if active
        if Path::new("/sys/fs/selinux/enforce").exists() {
            return MacFramework::SeLinux(SeLinuxInfo::read());
        }

        // AppArmor exposes itself under /sys/kernel/security/apparmor
        if Path::new("/sys/kernel/security/apparmor").exists() {
            return MacFramework::AppArmor(AppArmorInfo::read());
        }

        MacFramework::None
    }

    /// Returns true if something may actively block our tests.
    /// Unconfined contexts under either framework are considered safe.
    pub fn may_restrict(&self) -> bool {
        match self {
            MacFramework::SeLinux(info)  => info.enforcing && info.confined,
            MacFramework::AppArmor(info) => info.confined,
            MacFramework::None           => false,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            MacFramework::None => "none detected".into(),
            MacFramework::SeLinux(i) => format!(
                "SELinux {} | context: {}",
                if i.enforcing { "enforcing" } else { "permissive" },
                i.context,
            ),
            MacFramework::AppArmor(i) => format!(
                "AppArmor active | profile: {}",
                i.profile,
            ),
        }
    }
}

impl SeLinuxInfo {
    fn read() -> Self {
        let enforcing = fs::read_to_string("/sys/fs/selinux/enforce")
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        // /proc/self/attr/current holds our SELinux context, NUL-terminated
        let context = fs::read_to_string("/proc/self/attr/current")
            .map(|s| s.trim_end_matches('\0').trim().to_string())
            .unwrap_or_else(|_| "<unreadable>".into());

        // Context looks like: unconfined_u:unconfined_r:unconfined_t:s0-s0:c0.c1023
        // The type field (3rd colon-separated component) ending in _t tells us the domain.
        let confined = !context.contains("unconfined_t");

        SeLinuxInfo { enforcing, context, confined }
    }
}

impl AppArmorInfo {
    fn read() -> Self {
        // /proc/self/attr/current on AppArmor systems holds the profile name,
        // or the literal string "unconfined"
        let profile = fs::read_to_string("/proc/self/attr/current")
            .map(|s| s.trim_end_matches('\0').trim().to_string())
            .unwrap_or_else(|_| "<unreadable>".into());

        let confined = profile != "unconfined";

        AppArmorInfo { profile, confined }
    }
}

/// Check whether executing from the current binary's location is likely to be
/// blocked by SELinux's executions restrictions on tmp_t files.
///
/// This is the most common practical problem when copying binaries to /tmp.
pub fn check_exec_location() -> Option<String> {
    // Only relevant on SELinux hosts
    if !Path::new("/sys/fs/selinux/enforce").exists() {
        return None;
    }

    let exe = match fs::read_link("/proc/self/exe") {
        Ok(p) => p,
        Err(_) => return None,
    };

    let path_str = exe.to_string_lossy();

    // Warn if running from /tmp — SELinux often denies executing tmp_t files
    if path_str.starts_with("/tmp") {
        return Some(format!(
            "binary is running from {} — SELinux may label this tmp_t \
             which can prevent execution or restrict capabilities. \
             Consider copying to ~/musl-compat-test or /usr/local/bin/ instead.",
            path_str
        ));
    }

    None
}
