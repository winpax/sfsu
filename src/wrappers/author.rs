//! A wrapper around a git signature to display the author

use std::fmt::Display;

use sprinkles::git::{
    implementations::{git2, gix},
    parity::Signature,
};

#[must_use]
/// A wrapper around a git signature to display the author
pub struct Author {
    signature: Signature,
    show_emails: bool,
}

impl<'a> From<git2::Signature<'a>> for Author {
    fn from(signature: git2::Signature<'a>) -> Self {
        Self {
            signature: Signature::Git2(signature.to_owned()),
            show_emails: true,
        }
    }
}

impl<'a> From<gix::actor::SignatureRef<'a>> for Author {
    fn from(signature: gix::actor::SignatureRef<'a>) -> Self {
        Self {
            signature: Signature::Gitoxide(signature.into()),
            show_emails: true,
        }
    }
}

#[allow(dead_code)]
impl Author {
    /// Create a new author from the provided signature
    pub fn from_signature(signature: Signature) -> Self {
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

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl Display for Author {
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
