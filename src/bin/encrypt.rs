// encrypt — dev-time utility: plaintext sources → encrypted artifacts in repo
//
//   CV_SECRET=<secret> cargo run --bin encrypt
//
// Writes:
//   raw_history_plain.json → raw_history.json  (commit)
//   photo.png              → photo.enc          (commit; photo.png is gitignored)

use cv_engine::crypto;

fn main() -> anyhow::Result<()> {
    let secret = std::env::var("CV_SECRET").map_err(|_| {
        anyhow::anyhow!("CV_SECRET environment variable is required")
    })?;

    let plaintext = std::fs::read("raw_history_plain.json")
        .map_err(|_| anyhow::anyhow!("raw_history_plain.json not found (run from project root)"))?;
    let history_enc = crypto::encrypt(&plaintext, &secret)?;
    std::fs::write("raw_history.json", &history_enc)?;
    println!(
        "✓  {} bytes plaintext → raw_history.json ({} bytes)",
        plaintext.len(),
        history_enc.len()
    );

    match std::fs::read("photo.png") {
        Ok(photo) => {
            let photo_enc = crypto::encrypt(&photo, &secret)?;
            std::fs::write("photo.enc", &photo_enc)?;
            println!(
                "✓  {} bytes photo.png → photo.enc ({} bytes)",
                photo.len(),
                photo_enc.len()
            );
        }
        Err(_) => {
            println!("⚠  photo.png not found — skipped photo.enc (keep existing file if unchanged)");
        }
    }

    println!("   Commit raw_history.json and photo.enc.");
    println!("   Do NOT commit raw_history_plain.json or photo.png.");
    Ok(())
}
