use url::form_urlencoded;

pub fn encode_string(input: &str) -> String {
    form_urlencoded::byte_serialize(input.as_bytes()).collect()
}
