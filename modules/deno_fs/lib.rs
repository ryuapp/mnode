// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::{Ctx, Result as JsResult};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> JsResult<()> {
    // Ensure the internal symbol object and nested fs object exist
    ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')] ||= {}; globalThis[Symbol.for('mdeno.internal')].fs ||= {};")?;

    setup_internal(ctx).map_err(|e| {
        eprintln!("deno_fs setup_internal error: {}", e);
        rquickjs::Error::Unknown
    })?;

    ctx.eval::<(), _>(include_str!("deno_fs.js")).map_err(|e| {
        eprintln!("deno_fs.js eval error: {:?}", e);
        e
    })?;

    Ok(())
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
    // pathFromURLImpl(url: URL): string - Platform-specific URL to path conversion
    add_internal_function!(ctx, "pathFromURLImpl", |url_string: String| -> String {
        // Parse the URL object that was serialized as JSON
        // The JavaScript side sends us the pathname and hostname
        match serde_json::from_str::<serde_json::Value>(&url_string) {
            Ok(url_obj) => {
                let pathname = url_obj
                    .get("pathname")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let hostname = url_obj
                    .get("hostname")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Use platform-specific path conversion
                if cfg!(windows) {
                    path_from_url_win32(pathname, hostname)
                } else {
                    path_from_url_posix(pathname, hostname)
                }
            }
            Err(_) => String::new(),
        }
    });

    // readFileSync(path: string | URL): Uint8Array
    add_internal_function!(ctx, "fs.readFileSync", |path: String| -> Vec<u8> {
        match fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                // TODO: Return proper Deno error
                eprintln!("ReadFileSync error: {}", e);
                Vec::new()
            }
        }
    });

    // readTextFileSync(path: string | URL): string
    add_internal_function!(ctx, "fs.readTextFileSync", |path: String| -> String {
        match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("ReadTextFileSync error: {}", e);
                String::new()
            }
        }
    });

    // writeFileSync(path: string | URL, data: Uint8Array, options?: WriteFileOptions): void
    add_internal_function!(
        ctx,
        "fs.writeFileSync",
        |path: String, data: Vec<u8>, options: Option<String>| {
            let opts: Value = options
                .and_then(|o| serde_json::from_str(&o).ok())
                .unwrap_or(json!({}));

            let append = opts
                .get("append")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let create = opts.get("create").and_then(|v| v.as_bool()).unwrap_or(true);
            let create_new = opts
                .get("createNew")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if create_new && Path::new(&path).exists() {
                eprintln!("WriteFileSync error: File already exists");
                return;
            }

            let result = if append {
                let mut file = match fs::OpenOptions::new()
                    .create(create)
                    .append(true)
                    .open(&path)
                {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("WriteFileSync error: {}", e);
                        return;
                    }
                };
                use std::io::Write;
                file.write_all(&data)
            } else {
                fs::write(&path, &data)
            };

            if let Err(e) = result {
                eprintln!("WriteFileSync error: {}", e);
            }
        }
    );

    // writeTextFileSync(path: string | URL, text: string, options?: WriteFileOptions): void
    add_internal_function!(
        ctx,
        "fs.writeTextFileSync",
        |path: String, text: String, options: Option<String>| {
            let opts: Value = options
                .and_then(|o| serde_json::from_str(&o).ok())
                .unwrap_or(json!({}));

            let append = opts
                .get("append")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let create = opts.get("create").and_then(|v| v.as_bool()).unwrap_or(true);
            let create_new = opts
                .get("createNew")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if create_new && Path::new(&path).exists() {
                eprintln!("WriteTextFileSync error: File already exists");
                return;
            }

            let result = if append {
                let mut file = match fs::OpenOptions::new()
                    .create(create)
                    .append(true)
                    .open(&path)
                {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("WriteTextFileSync error: {}", e);
                        return;
                    }
                };
                use std::io::Write;
                file.write_all(text.as_bytes())
            } else {
                fs::write(&path, &text)
            };

            if let Err(e) = result {
                eprintln!("WriteTextFileSync error: {}", e);
            }
        }
    );

    // statSync(path: string | URL): FileInfo
    add_internal_function!(ctx, "fs.statSync", |path: String| -> String {
        match fs::metadata(&path) {
            Ok(metadata) => {
                let file_info = json!({
                    "isFile": metadata.is_file(),
                    "isDirectory": metadata.is_dir(),
                    "isSymlink": metadata.is_symlink(),
                    "size": metadata.len(),
                    "mtime": metadata.modified().ok().and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_millis())
                    }),
                    "atime": metadata.accessed().ok().and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_millis())
                    }),
                });
                file_info.to_string()
            }
            Err(e) => {
                eprintln!("StatSync error: {}", e);
                String::new()
            }
        }
    });

    // mkdirSync(path: string | URL, options?: MkdirOptions): void
    add_internal_function!(
        ctx,
        "fs.mkdirSync",
        |path: String, options: Option<String>| {
            let opts: Value = options
                .and_then(|o| serde_json::from_str(&o).ok())
                .unwrap_or(json!({}));

            let recursive = opts
                .get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let result = if recursive {
                fs::create_dir_all(&path)
            } else {
                fs::create_dir(&path)
            };

            if let Err(e) = result {
                eprintln!("MkdirSync error: {}", e);
            }
        }
    );

    // removeSync(path: string | URL, options?: RemoveOptions): void
    add_internal_function!(
        ctx,
        "fs.removeSync",
        |path: String, options: Option<String>| {
            let opts: Value = options
                .and_then(|o| serde_json::from_str(&o).ok())
                .unwrap_or(json!({}));

            let recursive = opts
                .get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let path_obj = Path::new(&path);
            let result = if !path_obj.exists() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Path not found",
                ))
            } else if path_obj.is_dir() {
                if recursive {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_dir(&path)
                }
            } else {
                fs::remove_file(&path)
            };

            if let Err(e) = result {
                eprintln!("RemoveSync error: {}", e);
            }
        }
    );

    // copyFileSync(fromPath: string | URL, toPath: string | URL): void
    add_internal_function!(ctx, "fs.copyFileSync", |from: String, to: String| {
        if let Err(e) = fs::copy(&from, &to) {
            eprintln!("CopyFileSync error: {}", e);
        }
    });

    Ok(())
}

// Helper function: Convert Windows file URL to path
// Matches Deno's pathFromURLWin32 implementation
fn path_from_url_win32(pathname: &str, hostname: &str) -> String {
    // Remove leading slashes and extract drive letter (e.g., /C:/ → C:/)
    let mut p = if let Some(rest) = pathname.strip_prefix('/') {
        if rest.len() >= 2 && rest.chars().nth(1) == Some(':') {
            // Drive letter format
            rest.to_string()
        } else {
            pathname.to_string()
        }
    } else {
        pathname.to_string()
    };

    // Replace forward slashes with backslashes
    p = p.replace('/', "\\");

    // Replace unescaped % with %25 (% not followed by two hex digits)
    let mut result = String::new();
    let chars: Vec<char> = p.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            // Check if followed by exactly two hex digits
            if i + 2 < chars.len() {
                let next_two: String = chars[i + 1..=i + 2].iter().collect();
                if next_two.chars().all(|c| c.is_ascii_hexdigit()) {
                    result.push('%');
                    i += 1;
                    continue;
                }
            }
            result.push_str("%25");
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Simple percent-decoding
    let mut decoded = String::new();
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex_str: String = chars[i + 1..=i + 2].iter().collect();
            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                decoded.push(byte as char);
                i += 3;
                continue;
            }
        }
        decoded.push(chars[i]);
        i += 1;
    }

    // Add hostname if present (UNC path)
    if !hostname.is_empty() {
        format!("\\\\{}{}", hostname, decoded)
    } else {
        decoded
    }
}

// Helper function: Convert POSIX file URL to path
// Matches Deno's pathFromURLPosix implementation
fn path_from_url_posix(pathname: &str, hostname: &str) -> String {
    // POSIX doesn't support host names in file URLs
    if !hostname.is_empty() {
        return String::new(); // Would be an error in real Deno
    }

    // Replace unescaped % with %25 (% not followed by two hex digits)
    let mut result = String::new();
    let chars: Vec<char> = pathname.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            // Check if followed by exactly two hex digits
            if i + 2 < chars.len() {
                let next_two: String = chars[i + 1..=i + 2].iter().collect();
                if next_two.chars().all(|c| c.is_ascii_hexdigit()) {
                    result.push('%');
                    i += 1;
                    continue;
                }
            }
            result.push_str("%25");
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Simple percent-decoding
    let mut decoded = String::new();
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex_str: String = chars[i + 1..=i + 2].iter().collect();
            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                decoded.push(byte as char);
                i += 3;
                continue;
            }
        }
        decoded.push(chars[i]);
        i += 1;
    }

    decoded
}
