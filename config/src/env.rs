use std::borrow::Cow;

#[must_use]
pub fn env_prefix(prefix: Option<&str>, suffix: &'static str) -> Cow<'static, str> {
    prefix.map_or(Cow::Borrowed(suffix), |s| Cow::Owned(format!("{s}_{suffix}")))
}
