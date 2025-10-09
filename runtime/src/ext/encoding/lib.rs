use crate::add_internal_function;
use rquickjs::Ctx;
use std::error::Error;

pub fn setup(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    ctx.eval::<(), _>("globalThis[Symbol.for('mnode.internal')].encoding = {};")?;

    // btoa: Binary to ASCII (Base64 encode)
    add_internal_function!(ctx, "encoding.btoa", |data: String| -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
    });

    // atob: ASCII to Binary (Base64 decode)
    add_internal_function!(ctx, "encoding.atob", |data: String| -> String {
        use base64::Engine;
        match base64::engine::general_purpose::STANDARD.decode(data.trim()) {
            Ok(decoded) => match String::from_utf8(decoded) {
                Ok(s) => s,
                Err(e) => format!("ERROR: Invalid UTF-8 sequence: {}", e),
            },
            Err(e) => format!("ERROR: Invalid base64 string: {}", e),
        }
    });

    // TextEncoder.encode: String to UTF-8 bytes (as array)
    add_internal_function!(ctx, "encoding.encode", |text: String| -> String {
        let bytes = text.into_bytes();
        serde_json::to_string(&bytes).unwrap()
    });

    // TextDecoder.decode: UTF-8 bytes to String
    add_internal_function!(ctx, "encoding.decode", |bytes_json: String| -> String {
        match serde_json::from_str::<Vec<u8>>(&bytes_json) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => format!("ERROR: Invalid UTF-8 sequence: {}", e),
            },
            Err(e) => format!("ERROR: Invalid bytes array: {}", e),
        }
    });

    Ok(())
}
