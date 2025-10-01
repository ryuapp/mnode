use crate::add_internal_function;
use rquickjs::Ctx;
use std::error::Error;
use url::Url;

pub fn setup(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    ctx.eval::<(), _>("globalThis[Symbol.for('mnode.internal')].url = {};")?;

    add_internal_function!(ctx, "url.parse", |url_str: String,
                                              base: String|
     -> String {
        parse_url(url_str, base)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    add_internal_function!(ctx, "url.setComponent", |url_str: String,
                                                     component: String,
                                                     value: String|
     -> String {
        set_url_component(url_str, component, value)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")))
    });

    Ok(())
}

pub fn parse_url(url_str: String, base: String) -> Result<String, String> {
    let parsed = if base.is_empty() {
        Url::parse(&url_str).map_err(|e| e.to_string())?
    } else {
        let base_url = Url::parse(&base).map_err(|e| e.to_string())?;
        base_url.join(&url_str).map_err(|e| e.to_string())?
    };

    let json = serde_json::json!({
        "href": parsed.as_str(),
        "origin": format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or("")),
        "protocol": format!("{}:", parsed.scheme()),
        "username": parsed.username(),
        "password": parsed.password().unwrap_or(""),
        "host": parsed.host_str().map(|h| {
            if let Some(port) = parsed.port() {
                format!("{}:{}", h, port)
            } else {
                h.to_string()
            }
        }).unwrap_or_default(),
        "hostname": parsed.host_str().unwrap_or(""),
        "port": parsed.port().map(|p| p.to_string()).unwrap_or_default(),
        "pathname": parsed.path(),
        "search": parsed.query().map(|q| format!("?{}", q)).unwrap_or_default(),
        "hash": parsed.fragment().map(|f| format!("#{}", f)).unwrap_or_default(),
    });

    Ok(serde_json::to_string(&json).unwrap())
}

pub fn set_url_component(
    url_str: String,
    component: String,
    value: String,
) -> Result<String, String> {
    let mut parsed = Url::parse(&url_str).map_err(|e| e.to_string())?;

    match component.as_str() {
        "protocol" => {
            let scheme = value.trim_end_matches(':');
            parsed
                .set_scheme(scheme)
                .map_err(|_| "Invalid protocol".to_string())?;
        }
        "username" => {
            parsed
                .set_username(&value)
                .map_err(|_| "Invalid username".to_string())?;
        }
        "password" => {
            parsed
                .set_password(Some(&value))
                .map_err(|_| "Invalid password".to_string())?;
        }
        "host" => {
            parsed.set_host(Some(&value)).map_err(|e| e.to_string())?;
        }
        "hostname" => {
            parsed.set_host(Some(&value)).map_err(|e| e.to_string())?;
        }
        "port" => {
            if value.is_empty() {
                parsed
                    .set_port(None)
                    .map_err(|_| "Invalid port".to_string())?;
            } else {
                let port: u16 = value.parse().map_err(|_| "Invalid port".to_string())?;
                parsed
                    .set_port(Some(port))
                    .map_err(|_| "Invalid port".to_string())?;
            }
        }
        "pathname" => {
            parsed.set_path(&value);
        }
        "search" => {
            let query = value.trim_start_matches('?');
            if query.is_empty() {
                parsed.set_query(None);
            } else {
                parsed.set_query(Some(query));
            }
        }
        "hash" => {
            let fragment = value.trim_start_matches('#');
            if fragment.is_empty() {
                parsed.set_fragment(None);
            } else {
                parsed.set_fragment(Some(fragment));
            }
        }
        _ => return Err("Unknown component".to_string()),
    }

    let json = serde_json::json!({
        "href": parsed.as_str(),
        "origin": format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or("")),
        "protocol": format!("{}:", parsed.scheme()),
        "username": parsed.username(),
        "password": parsed.password().unwrap_or(""),
        "host": parsed.host_str().map(|h| {
            if let Some(port) = parsed.port() {
                format!("{}:{}", h, port)
            } else {
                h.to_string()
            }
        }).unwrap_or_default(),
        "hostname": parsed.host_str().unwrap_or(""),
        "port": parsed.port().map(|p| p.to_string()).unwrap_or_default(),
        "pathname": parsed.path(),
        "search": parsed.query().map(|q| format!("?{}", q)).unwrap_or_default(),
        "hash": parsed.fragment().map(|f| format!("#{}", f)).unwrap_or_default(),
    });

    Ok(serde_json::to_string(&json).unwrap())
}
