/// Match the public waitlist's ASCII email policy before accessing D1.
pub fn normalize(input: &str) -> Option<String> {
    let email = input.trim().to_ascii_lowercase();
    let (local, domain) = email.split_once('@')?;
    if email.len() > 254
        || local.is_empty()
        || local.len() > 64
        || local.starts_with('.')
        || local.ends_with('.')
        || local.contains("..")
        || !local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+/=?^_`{|}~-".contains(&b))
        || !domain.contains('.')
        || !domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return None;
    }
    Some(email)
}
