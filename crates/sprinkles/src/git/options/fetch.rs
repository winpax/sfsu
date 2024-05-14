//! An opinionated [`git2::FetchOptions`] wrapper

use git2::{Progress, ProxyOptions, RemoteCallbacks};

use crate::{config, contexts::ScoopContext};

#[derive(Default)]
/// An opinionated [`git2::FetchOptions`] wrapper
pub struct FetchOptions<'a> {
    callbacks: Option<RemoteCallbacks<'a>>,
    proxy: Option<ProxyOptions<'a>>,

    /// This is used internally for [`Deref`] and [`DerefMut`]
    ///
    /// This should always be updated before being provided to the user
    __internal: git2::FetchOptions<'a>,
}

impl<'a> FetchOptions<'a> {
    #[must_use]
    /// Create new [`FetchOptions`] with default values
    pub fn new(ctx: &impl ScoopContext<config::Scoop>) -> Self {
        let mut this = Self::default();

        if let Some(proxy) = ctx.config().proxy.clone() {
            this.proxy(proxy);
        }

        this
    }

    /// Set the progress callbacks for the fetch operation
    pub fn transfer_progress(&mut self, progress: impl FnMut(Progress<'_>) -> bool + 'a) {
        let callbacks = self.callbacks.get_or_insert_with(RemoteCallbacks::default);
        callbacks.transfer_progress(progress);
    }

    /// Set the proxy for the fetch operation
    pub fn proxy(&mut self, proxy: impl Into<ProxyOptions<'a>>) {
        self.proxy = Some(proxy.into());
    }

    #[must_use]
    /// Convert the [`FetchOptions`] into a [`git2::FetchOptions`]
    pub fn as_git2<'b>(&'b mut self) -> &'b git2::FetchOptions<'a> {
        let options = &mut self.__internal;

        if let Some(callbacks) = self.callbacks.take() {
            options.remote_callbacks(callbacks);
        }

        if let Some(proxy) = self.proxy.take() {
            options.proxy_options(proxy);
        }

        &self.__internal
    }

    #[must_use]
    /// Convert the [`FetchOptions`] into a [`git2::FetchOptions`] and return a mutable reference to it
    pub fn as_git2_mut(&mut self) -> &mut git2::FetchOptions<'a> {
        _ = self.as_git2();

        &mut self.__internal
    }
}
