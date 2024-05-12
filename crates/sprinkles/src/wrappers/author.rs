//! A wrapper around a git signature to display the author

use std::fmt::Display;

use derive_more::Deref;

use crate::git::parity::Signature;

#[derive(Deref)]
#[must_use]
/// A wrapper around a git signature to display the author
pub struct Author<'a> {
    #[deref]
    signature: Signature<'a>,
    show_emails: bool,
}

impl<'a> From<git2::Signature<'a>> for Author<'a> {
    fn from(signature: git2::Signature<'a>) -> Self {
        Self {
            signature: Signature::Git2(signature),
            show_emails: true,
        }
    }
}

impl<'a> From<gix::actor::SignatureRef<'a>> for Author<'a> {
    fn from(signature: gix::actor::SignatureRef<'a>) -> Self {
        Self {
            signature: Signature::Gitoxide(signature),
            show_emails: true,
        }
    }
}

impl<'a> Author<'a> {
    /// Create a new author from the provided signature
    pub fn from_signature(signature: Signature<'a>) -> Self {
        Self {
            signature,
            show_emails: true,
        }
    }

    /// Apply whether to show emails to the [`Author`]
    pub fn with_show_emails(mut self, show_emails: bool) -> Self {
        self.show_emails = show_emails;
        self
    }
}

impl<'a> Display for Author<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let user_name = match &self.signature {
            Signature::Git2(sig) => sig.name().unwrap_or("Non-utf8 name").to_string(),
            Signature::Gitoxide(sig) => sig.name.to_string(),
        };

        let email = match &self.signature {
            Signature::Git2(sig) => sig.email().unwrap_or("No email").to_string(),
            Signature::Gitoxide(sig) => sig.email.to_string(),
        };

        user_name.fmt(f)?;

        if self.show_emails {
            write!(f, " <{email}>")?;
        }

        Ok(())
    }
}
