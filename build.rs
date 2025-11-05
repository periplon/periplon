use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Ensure web/out directory exists for RustEmbed when building with server feature
    // This is needed for docs.rs builds and when the web UI hasn't been built yet

    // Check if the server feature is enabled by looking at CARGO_FEATURE_SERVER env var
    let has_server_feature = env::var("CARGO_FEATURE_SERVER").is_ok();

    if has_server_feature {
        let web_out_path = PathBuf::from("web").join("out");

        if !web_out_path.exists() {
            // Create the directory structure
            fs::create_dir_all(&web_out_path)
                .expect("Failed to create web/out directory for RustEmbed");
        }

        // Check if index.html exists, if not create a minimal one
        let index_html_path = web_out_path.join("index.html");
        if !index_html_path.exists() {
            let minimal_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Periplon Web UI</title>
</head>
<body>
    <div style="display: flex; align-items: center; justify-content: center; height: 100vh; font-family: system-ui, -apple-system, sans-serif;">
        <div style="text-align: center;">
            <h1>Periplon Web UI</h1>
            <p>The web UI assets have not been built yet.</p>
            <p>To build the web UI, run:</p>
            <pre style="background: #f5f5f5; padding: 1em; border-radius: 4px;">cd web && npm install && npm run build</pre>
        </div>
    </div>
</body>
</html>"#;
            fs::write(&index_html_path, minimal_html).expect("Failed to create minimal index.html");
            eprintln!("Created minimal index.html for web UI (web assets not built)");
        }
    }

    // Re-run build script if web/out changes
    println!("cargo:rerun-if-changed=web/out");
}
