pub fn load_console() -> &'static str {
    include_str!("console/console.js")
}

pub fn load_fetch() -> &'static str {
    include_str!("fetch/fetch.js")
}

pub fn load_navigator() -> &'static str {
    include_str!("navigator/navigator.js")
}

pub fn load_url() -> &'static str {
    include_str!("url/url.js")
}
