use regex::Regex;

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

#[derive(Debug, Clone)]
pub struct StringMatch {
    text: String,
    partial: bool,
    case_sensitive: bool,
}

impl<S> From<S> for StringMatch
where
    S: Into<String>,
{
    fn from(text: S) -> Self {
        Self {
            text: text.into(),
            partial: false,
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

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn partial(mut self) -> Self {
        self.partial = true;
        self
    }

    pub fn whole(mut self) -> Self {
        self.partial = false;
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

impl Needle for StringMatch {
    fn is_match(&self, haystack: &str) -> bool {
        match self.case_sensitive {
            true => match self.partial {
                true => haystack.contains(&self.text),
                false => haystack == self.text,
            },
            false => match self.partial {
                true => haystack.to_lowercase().contains(&self.text.to_lowercase()),
                false => haystack.to_lowercase() == self.text.to_lowercase(),
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringmatch() {
        assert!(!StringMatch::from("a").is_partial());
        assert!(StringMatch::from("a").is_case_sensitive());
        assert!(StringMatch::from("a").is_match("a"));
        assert!(!StringMatch::from("a").is_match(""));
        assert!(!StringMatch::from("a").is_match("b"));
        assert!(!StringMatch::from("a").is_match("A"));

        assert!(StringMatch::from("a").partial().is_partial());
        assert!(StringMatch::from("a").partial().is_case_sensitive());
        assert!(StringMatch::from("a").partial().is_match("a"));
        assert!(StringMatch::from("a").partial().is_match("aa"));
        assert!(StringMatch::from("a").partial().is_match("dad"));
        assert!(StringMatch::from("a").partial().is_match("ba"));

        assert!(!StringMatch::from("a").case_insensitive().is_case_sensitive());
        assert!(!StringMatch::from("a").case_insensitive().is_partial());
        assert!(StringMatch::from("a").case_insensitive().is_match("a"));
        assert!(StringMatch::from("a").case_insensitive().is_match("A"));
        assert!(!StringMatch::from("a").case_insensitive().is_match("aa"));

        assert!(StringMatch::from("a").partial().case_insensitive().is_partial());
        assert!(!StringMatch::from("a").partial().case_insensitive().is_case_sensitive());
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("a"));
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("aa"));
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("A"));
        assert!(StringMatch::from("a").partial().case_insensitive().is_match("AA"));
        assert!(StringMatch::from("aA").partial().case_insensitive().is_match("Aa"));
        assert!(StringMatch::from("aA").partial().case_insensitive().is_match("Aaa"));
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
    }
}
