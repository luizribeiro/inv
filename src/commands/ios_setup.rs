use crate::error::Result;

pub fn run(url_override: Option<String>) -> Result<()> {
    let url = crate::config::resolve_ios_shortcut_url(url_override)?;
    let terminal_qr = crate::terminal_qr::render(&url)?;

    println!("iOS Shortcut Setup");
    println!("1. Open Camera on your iPhone and scan the QR code below.");
    println!("2. Tap the banner and install the Shortcut.");
    println!("3. Run the Shortcut and allow requested permissions.");
    println!();
    println!("{terminal_qr}");
    println!("Shortcut URL: {url}");

    Ok(())
}
