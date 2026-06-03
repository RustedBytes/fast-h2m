use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nightly)");

    let Ok(output) = Command::new("rustc").arg("-Vv").output() else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let version = String::from_utf8_lossy(&output.stdout);
    if version.lines().any(|line| {
        line.strip_prefix("release: ")
            .is_some_and(|release| release.contains("nightly"))
    }) {
        println!("cargo:rustc-cfg=nightly");
    }
}
