/// Returns the system root prefix from $MYCEL_ROOT, or "" for the live system.
/// Used during bootstrap to install into a target rootfs instead of the host.
pub fn system_root() -> String {
    std::env::var("MYCEL_ROOT").unwrap_or_default()
}
