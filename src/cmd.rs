use std::process::Command;

pub fn capture_screenshot(workspace_name: String) {
    log::debug!("Capturing screenshot of workspace: {}", workspace_name);
    let filename = format!("/tmp/{}.png", workspace_name);
    Command::new("rm")
        .arg(&filename)
        .output()
        .expect("Failed to remove screenshot");
    Command::new("scrot")
        .arg(&filename)
        .output()
        .expect("Failed to capture screenshot");
    log::debug!("Screenshot saved to {}", filename);
}