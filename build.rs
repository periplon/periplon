use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Ensure web/out directory exists for RustEmbed when building with server feature
    // This is needed for docs.rs builds and when the web UI hasn't been built yet

    // Check if the server feature is enabled by looking at CARGO_FEATURE_SERVER env var
    let has_server_feature = env::var("CARGO_FEATURE_SERVER").is_ok();

    if has_server_feature {
        let web_out_path = Path::new("web/out");
        if !web_out_path.exists() {
            // Create the directory structure
            fs::create_dir_all(web_out_path)
                .expect("Failed to create web/out directory for RustEmbed");

            // Create a placeholder file so the directory isn't empty
            let placeholder = web_out_path.join(".placeholder");
            fs::write(
                &placeholder,
                "This is a placeholder file created by build.rs to satisfy RustEmbed requirements.",
            )
            .expect("Failed to create placeholder file");

            println!("cargo:warning=Created web/out directory with placeholder for RustEmbed");
        }
    }

    // Re-run build script if web/out changes
    println!("cargo:rerun-if-changed=web/out");
}
