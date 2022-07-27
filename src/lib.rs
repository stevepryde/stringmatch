use regex::Regex;
#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};

pub trait Needle {
    fn is_match(&self, haystack: &str) -> bool;
}

pub trait NeedleIter: Needle {
    fn is_match_in<'a, I, S>(&self, haystacks: &mut I) -> bool
    where
        I: Iterator<Item = S>,
        S: Into<&'a str>,
    {
        haystacks.any(|h| self.is_match(h.into()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub enum StringMatchLength {
    /// Needle string must match the whole haystack string.
    Full,
    /// Needle string can be any substring within the haystack string.
    Partial,
    /// Needle string will only match strings within the haystack surrounded by spaces or
    /// a string boundary.
    Word,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct StringMatch {
    text: String,
    /// The match length to use. Default is StringMatchLength::Full, which means the needle
    /// string must match the entire haystack.
    match_length: StringMatchLength,
    /// If true, use a case-sensitive match. Default is true.
    case_sensitive: bool,
}

impl<S> From<S> for StringMatch
where
    S: Into<String>,
{
    fn from(text: S) -> Self {
        Self {
            text: text.into(),
            match_length: StringMatchLength::Full,
            case_sensitive: true,
        }
    }
}

impl StringMatch {
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::from(text)
    }

    pub fn is_full_match(&self) -> bool {
        matches!(self.match_length, StringMatchLength::Full)
    }

    pub fn is_partial_match(&self) -> bool {
        matches!(self.match_length, StringMatchLength::Partial)
    }

    pub fn is_word_match(&self) -> bool {
        matches!(self.match_length, StringMatchLength::Word)
    }

    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn partial(mut self) -> Self {
        self.match_length = StringMatchLength::Partial;
        self
    }

    pub fn full(mut self) -> Self {
        self.match_length = StringMatchLength::Full;
        self
    }

    pub fn word(mut self) -> Self {
        self.match_length = StringMatchLength::Word;
        self
    }

    pub fn case_insensitive(mut self) -> Self {
        self.case_sensitive = false;
        self
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }
}

fn needle_in_haystack(needle: &str, haystack: &str, match_length: &StringMatchLength) -> bool {
    match match_length {
        StringMatchLength::Full => haystack == needle,
        StringMatchLength::Partial => haystack.contains(needle),
        StringMatchLength::Word => format!(" {} ", haystack).contains(&format!(" {} ", needle)),
    }
}

impl Needle for StringMatch {
    fn is_match(&self, haystack: &str) -> bool {
        match self.case_sensitive {
            true => needle_in_haystack(&self.text, haystack, &self.match_length),
            false => {
                let hs = haystack.to_lowercase();
                let needle = self.text.to_lowercase();
                needle_in_haystack(&needle, &hs, &self.match_length)
            }
        }
    }
}

impl Needle for Regex {
    fn is_match(&self, haystack: &str) -> bool {
        self.is_match(haystack)
    }
}

impl Needle for &str {
    fn is_match(&self, haystack: &str) -> bool {
        self == &haystack
    }
}

impl Needle for String {
    fn is_match(&self, haystack: &str) -> bool {
        self == haystack
    }
}

impl<F> Needle for F
where
    F: Fn(&str) -> bool,
{
    fn is_match(&self, haystack: &str) -> bool {
        self(haystack)
    }
}

pub trait StringMatchable: Into<StringMatch> {
    fn match_case_sensitive(self) -> StringMatch {
        self.into().case_sensitive()
    }

    fn match_case_insensitive(self) -> StringMatch {
        self.into().case_insensitive()
    }

    fn match_full(self) -> StringMatch {
        self.into().full()
    }

    fn match_word(self) -> StringMatch {
        self.into().word()
    }

    fn match_partial(self) -> StringMatch {
        self.into().partial()
    }
}

impl StringMatchable for String {}
impl StringMatchable for &str {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringmatch() {
        assert!(StringMatch::from("a").is_full_match());
        assert!(!StringMatch::from("a").is_partial_match());
        assert!(StringMatch::from("a").is_case_sensitive());
        assert!(StringMatch::from("a").is_match("a"));
        assert!(!StringMatch::from("a").is_match(""));
        assert!(!StringMatch::from("a").is_match("b"));
        assert!(!StringMatch::from("a").is_match("A"));

        assert!(StringMatch::from("a").partial().is_partial_match());
        assert!(!StringMatch::from("a").partial().is_full_match());
        assert!(!StringMatch::from("a").partial().is_word_match());
        assert!(StringMatch::from("a").partial().is_case_sensitive());
        assert!(StringMatch::from("a").partial().is_match("a"));
        assert!(StringMatch::from("a").partial().is_match("aa"));
        assert!(StringMatch::from("a").partial().is_match("dad"));
        assert!(StringMatch::from("a").partial().is_match("ba"));
        assert!(!StringMatch::from("A").partial().is_match("a"));
        assert!(!StringMatch::from("a").partial().is_match("A"));

        assert!(!StringMatch::from("a").case_insensitive().is_case_sensitive());
        assert!(!StringMatch::from("a").case_insensitive().is_partial_match());
        assert!(StringMatch::from("a").case_insensitive().is_match("a"));
        assert!(StringMatch::from("a").case_insensitive().is_match("A"));
        assert!(!StringMatch::from("a").case_insensitive().is_match("aa"));

        assert!(StringMatch::from("a").partial().case_insensitive().is_partial_match());
        assert!(!StringMatch::from("a").partial().case_insensitive().is_case_sensitive());
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("a"));
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("aa"));
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("A"));
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("AA"));
        assert!(StringMatch::from("aA").partial().case_insensitive().is_match("Aa"));
        assert!(StringMatch::from("aA").partial().case_insensitive().is_match("Aaa"));

        assert!(StringMatch::from("a").word().is_word_match());
        assert!(!StringMatch::from("a").word().is_partial_match());
        assert!(!StringMatch::from("a").word().is_full_match());
        assert!(StringMatch::from("a").word().is_case_sensitive());
        assert!(StringMatch::from("a").word().is_match("a"));
        assert!(!StringMatch::from("a").word().is_match("aa"));
        assert!(!StringMatch::from("a").word().is_match("dad"));
        assert!(!StringMatch::from("a").word().is_match("ba"));
        assert!(StringMatch::from("a").word().is_match("aa a aa"));
        assert!(StringMatch::from("a").word().is_match("a aa"));
        assert!(StringMatch::from("a").word().is_match("aa a"));
        assert!(!StringMatch::from("A").word().is_match("a"));
        assert!(!StringMatch::from("a").word().is_match("A"));
        assert!(StringMatch::from("aaa aa").word().case_insensitive().is_match("aaa aa"));
        assert!(StringMatch::from("aaa aa").word().case_insensitive().is_match("aa aaa aa aaa"));
        assert!(StringMatch::from("aaa aa").word().case_insensitive().is_match("aaa aa aaa"));
        assert!(StringMatch::from("aaa aa").word().case_insensitive().is_match("aa aaa aa"));
        assert!(!StringMatch::from("aaa aa").word().case_insensitive().is_match("aa aaa aaa"));

        assert!(StringMatch::from("a").word().case_insensitive().is_word_match());
        assert!(!StringMatch::from("a").word().case_insensitive().is_case_sensitive());
        assert!(StringMatch::from("a").word().case_insensitive().is_match("a"));
        assert!(!StringMatch::from("a").word().case_insensitive().is_match("aa"));
        assert!(StringMatch::from("a").word().case_insensitive().is_match("A"));
        assert!(StringMatch::from("a").word().case_insensitive().is_match("AA A AA"));
        assert!(StringMatch::from("aA").word().case_insensitive().is_match("Aa"));
        assert!(StringMatch::from("aA").word().case_insensitive().is_match("aa"));
        assert!(StringMatch::from("A").word().case_insensitive().is_match("aa a aa"));
        assert!(StringMatch::from("A").word().case_insensitive().is_match("a aa"));
        assert!(StringMatch::from("A").word().case_insensitive().is_match("aa a"));
        assert!(StringMatch::from("AAA AA").word().case_insensitive().is_match("aaa aa"));
        assert!(StringMatch::from("AAA AA").word().case_insensitive().is_match("aa aaa aa aaa"));
        assert!(StringMatch::from("AAA AA").word().case_insensitive().is_match("aaa aa aaa"));
        assert!(StringMatch::from("AAA AA").word().case_insensitive().is_match("aa aaa aa"));
        assert!(!StringMatch::from("AAA AA").word().case_insensitive().is_match("aa aaa aaa"));
    }

    #[test]
    fn test_stringmatchable() {
        assert_eq!("a".match_full(), StringMatch::new("a").full());
        assert_eq!("a".match_partial(), StringMatch::new("a").partial());
        assert_eq!("a".match_word(), StringMatch::new("a").word());
        assert_eq!("a".match_case_insensitive(), StringMatch::new("a").case_insensitive());
        assert_eq!("a".match_case_sensitive(), StringMatch::new("a").case_sensitive());

        assert_eq!(String::from("a").match_full(), StringMatch::new("a").full());
        assert_eq!(String::from("a").match_partial(), StringMatch::new("a").partial());
        assert_eq!(String::from("a").match_word(), StringMatch::new("a").word());
        assert_eq!(
            String::from("a").match_case_insensitive(),
            StringMatch::new("a").case_insensitive()
        );
        assert_eq!(
            String::from("a").match_case_sensitive(),
            StringMatch::new("a").case_sensitive()
        );
    }

    fn needle_is_match<N>(needle: N) -> bool
    where
        N: Needle,
    {
        needle.is_match("Test")
    }

    #[test]
    fn test_needle() {
        assert!(needle_is_match("Test"));
        assert!(!needle_is_match("test")); // Strings are case-sensitive.
        assert!(!needle_is_match("Te")); // Strings always match whole haystack.

        assert!(needle_is_match(String::from("Test")));
        assert!(!needle_is_match(String::from("test"))); // Strings are case-sensitive.
        assert!(!needle_is_match(String::from("Te"))); // Strings always match whole haystack.

        assert!(needle_is_match(StringMatch::from("Test")));
        assert!(!needle_is_match(StringMatch::from("test")));
        assert!(needle_is_match(StringMatch::from("test").case_insensitive()));
        assert!(needle_is_match(StringMatch::from("Te").partial()));
        assert!(!needle_is_match(StringMatch::from("te").partial()));
        assert!(needle_is_match(StringMatch::from("te").partial().case_insensitive()));

        assert!(needle_is_match(Regex::new("Test").unwrap()));
        assert!(needle_is_match(Regex::new("Te").unwrap())); // Regex is partial by default unless ^$ specified.
        assert!(!needle_is_match(Regex::new("te").unwrap())); // Regex is case-sensitive by default.
        assert!(needle_is_match(Regex::new(r"(?i)te").unwrap())); // Case insensitive.
        assert!(needle_is_match(Regex::new(r"\w+").unwrap()));
        assert!(needle_is_match(Regex::new(r"\w").unwrap()));
        assert!(!needle_is_match(Regex::new(r"^T$").unwrap()));
        assert!(!needle_is_match(Regex::new(r"^est").unwrap()));
        assert!(!needle_is_match(Regex::new(r"Te$").unwrap()));
        assert!(needle_is_match(Regex::new(r"^T.+t$").unwrap()));
    }

    fn dynamic_dispatched_needle(needle: &dyn Needle) -> bool {
        needle.is_match("Test")
    }

    #[test]
    fn test_dyn_needle() {
        assert!(dynamic_dispatched_needle(&"Test"));
        assert!(!dynamic_dispatched_needle(&"test")); // Strings are case-sensitive.
        assert!(!dynamic_dispatched_needle(&"Te")); // Strings always match whole haystack.

        assert!(dynamic_dispatched_needle(&String::from("Test")));
        assert!(!dynamic_dispatched_needle(&String::from("test"))); // Strings are case-sensitive.
        assert!(!dynamic_dispatched_needle(&String::from("Te"))); // Strings always match whole haystack.

        assert!(dynamic_dispatched_needle(&StringMatch::from("Test")));
        assert!(!dynamic_dispatched_needle(&StringMatch::from("test")));
        assert!(dynamic_dispatched_needle(&StringMatch::from("test").case_insensitive()));
        assert!(dynamic_dispatched_needle(&StringMatch::from("Te").partial()));
        assert!(!dynamic_dispatched_needle(&StringMatch::from("te").partial()));
        assert!(dynamic_dispatched_needle(&StringMatch::from("te").partial().case_insensitive()));

        assert!(dynamic_dispatched_needle(&Regex::new("Test").unwrap()));
        assert!(dynamic_dispatched_needle(&Regex::new("Te").unwrap())); // Regex is partial by default unless ^$ specified.
        assert!(!dynamic_dispatched_needle(&Regex::new("te").unwrap())); // Regex is case-sensitive by default.
        assert!(dynamic_dispatched_needle(&Regex::new(r"(?i)te").unwrap())); // Case insensitive.
        assert!(dynamic_dispatched_needle(&Regex::new(r"\w+").unwrap()));
        assert!(dynamic_dispatched_needle(&Regex::new(r"\w").unwrap()));
        assert!(!dynamic_dispatched_needle(&Regex::new(r"^T$").unwrap()));
        assert!(!dynamic_dispatched_needle(&Regex::new(r"^est").unwrap()));
        assert!(!dynamic_dispatched_needle(&Regex::new(r"Te$").unwrap()));
        assert!(dynamic_dispatched_needle(&Regex::new(r"^T.+t$").unwrap()));

        assert!(dynamic_dispatched_needle(&|s: &str| s == "Test"));
        assert!(!dynamic_dispatched_needle(&|s: &str| s == "test"));
        assert!(!dynamic_dispatched_needle(&|s: &str| s == "Te"));
    }

    #[cfg(feature = "serde_derive")]
    #[test]
    fn test_serde() {
        let orig = StringMatch::new("a").partial().case_insensitive();
        let serialized: String = serde_json::to_string(&orig).unwrap();
        let deserialized: StringMatch = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, orig);
    }
}
