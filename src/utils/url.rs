use url::form_urlencoded;

pub fn encode_string(input: &str) -> String {
    form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

pub fn decode_string(input: &str) -> String {
    form_urlencoded::parse(input.as_bytes())
        .map(|(key, _)| key)
        .collect()
}
