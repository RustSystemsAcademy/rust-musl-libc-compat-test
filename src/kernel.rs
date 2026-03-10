use std::ffi::CStr;

/// Parsed kernel version for runtime ABI gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl KernelVersion {
    /// Query the running kernel via `uname(2)`.
    pub fn current() -> Self {
        let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
        unsafe { libc::uname(&mut uts) };
        let release = unsafe { CStr::from_ptr(uts.release.as_ptr()) }
            .to_string_lossy();
        Self::parse(&release)
    }

    /// Parse a version string like "5.14.0-362.el9" or "3.12.49-11-default".
    /// Splits on any non-digit character and takes the first three numeric tokens.
    pub fn parse(s: &str) -> Self {
        let mut nums = s
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .take(3)
            .map(|s| s.parse::<u32>().unwrap_or(0));

        Self {
            major: nums.next().unwrap_or(0),
            minor: nums.next().unwrap_or(0),
            patch: nums.next().unwrap_or(0),
        }
    }

    /// Returns true if the running kernel is at least `major.minor`.
    pub fn at_least(&self, major: u32, minor: u32) -> bool {
        *self >= Self { major, minor, patch: 0 }
    }
}

impl std::fmt::Display for KernelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rhel_style() {
        let v = KernelVersion::parse("5.14.0-362.8.1.el9.x86_64");
        assert_eq!(v, KernelVersion { major: 5, minor: 14, patch: 0 });
    }

    #[test]
    fn parse_suse_style() {
        let v = KernelVersion::parse("3.12.49-11-default");
        assert_eq!(v, KernelVersion { major: 3, minor: 12, patch: 49 });
    }

    #[test]
    fn ordering() {
        let old = KernelVersion::parse("3.12.0");
        let new = KernelVersion::parse("5.14.0");
        assert!(new > old);
        assert!(old.at_least(3, 12));
        assert!(!old.at_least(3, 17));
    }
}
