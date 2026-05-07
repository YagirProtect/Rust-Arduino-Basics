//! Lightweight flow helpers for `Option` and `Result`.
//!
//! These methods are useful in `no_std` code when you want to attach side effects
//! without breaking expression chains.

/// Extra callbacks for `Option<T>`.
pub trait OptionExt<T> {
    /// Run `f` when value is `Some`, then return the original option.
    fn on_some<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);

    /// Run `f` when value is `None`, then return the original option.
    fn on_none<F>(self, f: F) -> Self
    where
        F: FnOnce();
}

impl<T> OptionExt<T> for Option<T> {
    fn on_some<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Some(ref value) = self {
            f(value);
        }
        self
    }

    fn on_none<F>(self, f: F) -> Self
    where
        F: FnOnce(),
    {
        if self.is_none() {
            f();
        }
        self
    }
}

/// Extra callbacks for `Result<T, E>`.
pub trait ResultExt<T, E> {
    /// Run `f` when result is `Ok`, then return the original result.
    fn on_ok<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);

    /// Run `f` when result is `Err`, then return the original result.
    fn on_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E);
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn on_ok<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref value) = self {
            f(value);
        }
        self
    }

    fn on_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref err) = self {
            f(err);
        }
        self
    }
}
