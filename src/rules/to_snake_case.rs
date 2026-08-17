//! The name transform every shipping casing is built from.

/// The snake_case spelling of a name — `Credentials` becomes `credentials`,
/// `HTTPClient` becomes `http_client`.
pub fn to_snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (index, &current) in chars.iter().enumerate() {
        if current.is_uppercase() {
            let previous = if index == 0 {
                None
            } else {
                Some(chars[index - 1])
            };
            let next = chars.get(index + 1).copied();
            let boundary = match previous {
                None | Some('_') => false,
                // `userId` -> `user_id`, and `HTTPClient` -> `http_client`.
                Some(previous) => {
                    previous.is_lowercase()
                        || previous.is_numeric()
                        || next.is_some_and(char::is_lowercase)
                }
            };
            if boundary {
                out.push('_');
            }
            out.extend(current.to_lowercase());
        } else {
            out.push(current);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_handles_acronyms_and_digits() {
        for (input, expected) in [
            ("Credentials", "credentials"),
            ("authenticate", "authenticate"),
            ("HTTPClient", "http_client"),
            ("UserID", "user_id"),
            ("parse2Json", "parse2_json"),
            ("_Private", "_private"),
            ("already_snake", "already_snake"),
        ] {
            assert_eq!(to_snake_case(input), expected, "for `{input}`");
        }
    }
}
