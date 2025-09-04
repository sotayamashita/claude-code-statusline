use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Run a function with a timeout. Returns Ok(Some(T)) on success, Ok(None) on timeout.
/// If the task panics or returns an error, it is propagated as Err.
pub fn run_with_timeout<F, T>(dur: Duration, f: F) -> Result<Option<T>>
where
    F: Send + 'static + FnOnce() -> Result<T>,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        // Map panic into anyhow error; send result through channel if possible
        let _ = match res {
            Ok(Ok(val)) => tx.send(Ok(val)),
            Ok(Err(err)) => tx.send(Err(err)),
            Err(_) => tx.send(Err(anyhow::anyhow!("task panicked"))),
        };
    });

    match rx.recv_timeout(dur) {
        Ok(Ok(v)) => Ok(Some(v)),
        Ok(Err(e)) => Err(e),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(anyhow::anyhow!("worker disconnected")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_before_timeout() {
        let out = run_with_timeout(Duration::from_millis(200), || {
            std::thread::sleep(Duration::from_millis(50));
            Ok::<_, anyhow::Error>(42)
        })
        .unwrap();
        assert_eq!(out, Some(42));
    }

    #[test]
    fn returns_none_on_timeout() {
        let out = run_with_timeout(Duration::from_millis(30), || {
            std::thread::sleep(Duration::from_millis(100));
            Ok::<_, anyhow::Error>(99)
        })
        .unwrap();
        assert_eq!(out, None);
    }

    #[test]
    fn propagates_error() {
        let err = run_with_timeout(Duration::from_millis(100), || {
            Err::<i32, _>(anyhow::anyhow!("boom"))
        })
        .unwrap_err();
        assert!(format!("{err}").contains("boom"));
    }
}
