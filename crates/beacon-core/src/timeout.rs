//! Timeout execution utilities
//!
//! This module provides functions for running operations with time limits,
//! ensuring that slow operations don't block the status line generation.

use crate::error::CoreError;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Executes a function with a timeout constraint
///
/// Spawns the function in a separate thread and waits for completion
/// up to the specified duration. This prevents slow operations from
/// blocking the main thread indefinitely.
///
/// # Arguments
///
/// * `dur` - Maximum duration to wait for the function to complete
/// * `f` - The function to execute with timeout protection
///
/// # Returns
///
/// * `Ok(Some(T))` - Function completed successfully within timeout
/// * `Ok(None)` - Function timed out
/// * `Err` - Function panicked or returned an error
///
/// # Examples
///
/// ```
/// use beacon::timeout::run_with_timeout;
/// use std::time::Duration;
///
/// let result = run_with_timeout(Duration::from_millis(100), || {
///     Ok("Success".to_string())
/// });
/// assert!(result.unwrap().is_some());
///
/// let timeout = run_with_timeout(Duration::from_millis(10), || {
///     std::thread::sleep(Duration::from_millis(100));
///     Ok("Too slow")
/// });
/// assert!(timeout.unwrap().is_none());
/// ```
///
/// # Implementation Notes
///
/// - Uses channels for thread communication
/// - Catches panics and converts them to errors
/// - Thread is detached after timeout (may continue running)
pub fn run_with_timeout<F, T>(dur: Duration, f: F) -> Result<Option<T>, CoreError>
where
    F: Send + 'static + FnOnce() -> Result<T, CoreError>,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        // Map panic into typed error; send result through channel if possible
        let _ = match res {
            Ok(Ok(val)) => tx.send(Ok(val)),
            Ok(Err(err)) => tx.send(Err(err)),
            Err(_) => tx.send(Err(CoreError::TaskPanic)),
        };
    });

    match rx.recv_timeout(dur) {
        Ok(Ok(v)) => Ok(Some(v)),
        Ok(Err(e)) => Err(e),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(CoreError::WorkerDisconnected),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_before_timeout() {
        let out = run_with_timeout(Duration::from_millis(200), || {
            std::thread::sleep(Duration::from_millis(50));
            Ok::<_, CoreError>(42)
        })
        .unwrap();
        assert_eq!(out, Some(42));
    }

    #[test]
    fn returns_none_on_timeout() {
        let out = run_with_timeout(Duration::from_millis(30), || {
            std::thread::sleep(Duration::from_millis(100));
            Ok::<_, CoreError>(99)
        })
        .unwrap();
        assert_eq!(out, None);
    }

    #[test]
    fn propagates_error() {
        let err = run_with_timeout(Duration::from_millis(100), || {
            Err::<i32, _>(CoreError::InvalidConfig("boom".to_string()))
        })
        .unwrap_err();
        assert!(format!("{err}").contains("boom"));
    }
}
