use std::fmt::Display;

use derive_more::Deref;
use git2::Signature;

#[derive(Deref)]
#[must_use]
pub struct Author<'a> {
    #[deref]
    signature: Signature<'a>,
    show_emails: bool,
}

impl<'a> From<Signature<'a>> for Author<'a> {
    fn from(signature: Signature<'a>) -> Self {
        Self {
            signature,
            show_emails: true,
        }
    }
}

impl<'a> Author<'a> {
    pub fn from_signature(signature: Signature<'a>) -> Self {
        Self {
            signature,
            show_emails: true,
        }
    }

    pub fn with_show_emails(mut self, show_emails: bool) -> Self {
        self.show_emails = show_emails;
        self
    }
}

impl<'a> Display for Author<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let user_name = self.name().unwrap_or("No name");

        user_name.fmt(f)?;

        if self.show_emails {
            if let Some(user_email) = self.email() {
                write!(f, " <{user_email}>")?;
            }
        }

        Ok(())
    }
}
