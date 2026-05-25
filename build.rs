// DuckDB on Windows requires linking against rstrtmgr.lib (Windows Restart Manager API).
// Not needed on Linux — the Docker build won't need this.
fn main() {
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=rstrtmgr");
}
