use unicode_segmentation::UnicodeSegmentation;

const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

#[derive(Debug)]
pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty = s.trim().is_empty();

        let is_max_length = s.graphemes(true).count() > 256;

        let has_forbidden_characters = s.chars().any(|g| FORBIDDEN_CHARACTERS.contains(&g));

        if is_empty || is_max_length || has_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use super::{SubscriberName, FORBIDDEN_CHARACTERS};

    #[test]
    fn test_valid_name() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn test_256_long_name_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn test_more_than_256_long_is_invalid() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn test_only_whitespaces_is_invalid() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn test_empty_is_invalid() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn test_name_containing_invalid_characters_is_invalid() {
        for name in &FORBIDDEN_CHARACTERS {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
}
