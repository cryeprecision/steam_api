use serde::{Deserialize, Serialize};

/// Some respnses from the Steam API don't contain any data
/// because the profile is private, this enum is here
/// to reflect that concept.
///
/// But just using [`Option`] make much more sense, this is
/// not used anywhere.
#[derive(Serialize, Debug)]
pub enum Visibility<T> {
    /// The user has set his profile to private
    /// and we can't access the requested data.
    Hidden,
    /// The user has made the requested data
    /// publicly accessible.
    Visible(T),
}

impl<T> Visibility<T> {
    pub fn into_option(self) -> Option<T> {
        match self {
            Visibility::Hidden => None,
            Visibility::Visible(data) => Some(data),
        }
    }
    pub const fn as_option_ref(&self) -> Option<&T> {
        match self {
            Visibility::Hidden => None,
            Visibility::Visible(data) => Some(data),
        }
    }
    fn from_option(opt: Option<T>) -> Self {
        opt.map_or(Visibility::Hidden, Visibility::Visible)
    }
}

impl<'de, T> Deserialize<'de> for Visibility<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let option = <Option<T> as Deserialize<'de>>::deserialize(deserializer)?;
        Ok(Visibility::from_option(option))
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::Visibility;

    #[derive(Deserialize, Serialize)]
    struct Test {
        value: Visibility<u64>,
    }

    #[test]
    fn deserialize_some() {
        let json = serde_json::json!({ "value": 1 }).to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let option = parsed.value.into_option();
        assert_eq!(option, Some(1));
    }

    #[test]
    fn deserialize_none() {
        let json = serde_json::json!({}).to_string();

        let parsed: Test = serde_json::from_str(&json).unwrap();
        let option = parsed.value.into_option();
        assert_eq!(option, None);
    }
}
