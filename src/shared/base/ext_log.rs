use std::fmt::Display;

/// Extension trait for [`Result`] providing ergonomic tracing log helpers
/// without requiring manual closures or format strings.
pub trait ResultExt<T, E> {
    /// Logs the error at the `ERROR` level if the result is an [`Err`].
    ///
    /// Use this for critical failures that disrupt normal execution.
    fn log_err(self) -> Result<T, E>;

    /// Logs the error at the `WARN` level if the result is an [`Err`].
    ///
    /// Useful for non-fatal errors, recoverable fallbacks, or retryable attempts.
    fn log_warn(self) -> Result<T, E>;

    /// Logs the error at the `INFO` level if the result is an [`Err`].
    ///
    /// Suitable for expected failures or business-logic rejections that
    /// are not system errors.
    fn log_info(self) -> Result<T, E>;

    /// Logs the error at the `DEBUG` level if the result is an [`Err`].
    ///
    /// Ideal for diagnostic information during development or troubleshooting.
    fn log_debug(self) -> Result<T, E>;

    /// Logs the error at the `TRACE` level if the result is an [`Err`].
    ///
    /// Used for low-level, high-volume debugging information.
    fn log_trace(self) -> Result<T, E>;

    /// Logs the success value at the `INFO` level if the result is an [`Ok`].
    fn log_ok(self) -> Result<T, E>
    where
        T: Display;
}

impl<T, E: Display> ResultExt<T, E> for Result<T, E> {
    #[inline]
    fn log_err(self) -> Result<T, E> {
        self.inspect_err(|e| tracing::error!("{e}"))
    }

    #[inline]
    fn log_warn(self) -> Result<T, E> {
        self.inspect_err(|e| tracing::warn!("{e}"))
    }

    #[inline]
    fn log_info(self) -> Result<T, E> {
        self.inspect_err(|e| tracing::info!("{e}"))
    }

    #[inline]
    fn log_debug(self) -> Result<T, E> {
        self.inspect_err(|e| tracing::debug!("{e}"))
    }

    #[inline]
    fn log_trace(self) -> Result<T, E> {
        self.inspect_err(|e| tracing::trace!("{e}"))
    }

    #[inline]
    fn log_ok(self) -> Result<T, E>
    where
        T: Display,
    {
        self.inspect(|val| tracing::info!("{val}"))
    }
}
