//! Windows multimedia timer resolution.
//!
//! `TerminalView` coalesces backend wakeups behind a 4 ms window before it
//! paints (`terminal::view`). That wait is serviced by the Windows system
//! timer, whose default tick is 15.625 ms - so a window nominally worth 4 ms
//! closes somewhere between 4 ms and 16 ms depending on where in the tick it
//! opened. The added mean latency is unwelcome; the *spread* is worse, because
//! it lands each keystroke echo in a different frame and reads as a stutter
//! while typing even though no frame is genuinely slow to produce.
//!
//! Requesting a 1 ms period collapses that spread. Since Windows 10 version
//! 2004 the request is scoped to the calling process: a period held by some
//! other process does not help Paneflow, and Paneflow's own does not keep the
//! rest of the system awake. The returned guard restores the default.
//!
//! Linux and macOS honour sub-millisecond timer deadlines without asking, so
//! the guard is an inert placeholder there and the call site stays uncfg'd.

/// Set to `1` to skip the request and run on the default ~15.6 ms tick. Kept so
/// one binary can A/B its own frame pacing without a rebuild.
#[cfg(target_os = "windows")]
const DISABLE_ENV: &str = "PANEFLOW_NO_TIMER_BOOST";

/// 1 ms is the floor Windows accepts; the terminal's 4 ms window needs nothing
/// finer, and asking for less would only cost power for no pacing gain.
#[cfg(target_os = "windows")]
const TARGET_PERIOD_MS: u32 = 1;

/// `TIMERR_NOERROR` from `mmsyscom.h`.
#[cfg(target_os = "windows")]
const TIMERR_NOERROR: u32 = 0;

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(period: u32) -> u32;
    fn timeEndPeriod(period: u32) -> u32;
}

/// Holds the elevated timer period for as long as it is alive, and restores the
/// system default on drop.
pub(crate) struct TimerResolutionGuard {
    /// `Some` only while a period is actually held - `None` on non-Windows,
    /// when the kill switch is set, or when Windows refused the request.
    period_ms: Option<u32>,
}

impl TimerResolutionGuard {
    const fn inert() -> Self {
        Self { period_ms: None }
    }
}

impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        let Some(period_ms) = self.period_ms.take() else {
            return;
        };

        #[cfg(target_os = "windows")]
        {
            // SAFETY: pairs one-for-one with the `timeBeginPeriod` call that
            // produced this guard; `period_ms` is that same accepted value.
            let result = unsafe { timeEndPeriod(period_ms) };
            if result != TIMERR_NOERROR {
                log::debug!(
                    target: "paneflow::windows_timer",
                    "timeEndPeriod({period_ms}) returned MMRESULT {result}"
                );
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = period_ms;
    }
}

/// Request a 1 ms process timer period so the terminal's coalescing window
/// closes when it says it will. Hold the guard for the lifetime of the GUI.
#[cfg(target_os = "windows")]
pub(crate) fn boost_timer_resolution() -> TimerResolutionGuard {
    if std::env::var(DISABLE_ENV).as_deref() == Ok("1") {
        log::info!(
            target: "paneflow::windows_timer",
            "{DISABLE_ENV}=1: staying on the default ~15.6 ms timer tick"
        );
        return TimerResolutionGuard::inert();
    }

    // SAFETY: `timeBeginPeriod` takes a plain integer and carries no
    // precondition beyond a matching `timeEndPeriod`, which `Drop` performs.
    let result = unsafe { timeBeginPeriod(TARGET_PERIOD_MS) };
    if result == TIMERR_NOERROR {
        log::info!(
            target: "paneflow::windows_timer",
            "holding a {TARGET_PERIOD_MS} ms process timer period for terminal frame pacing"
        );
        return TimerResolutionGuard {
            period_ms: Some(TARGET_PERIOD_MS),
        };
    }

    log::warn!(
        target: "paneflow::windows_timer",
        "timeBeginPeriod({TARGET_PERIOD_MS}) was refused (MMRESULT {result}); terminal redraws \
         stay on the default ~15.6 ms tick"
    );
    TimerResolutionGuard::inert()
}

/// Linux and macOS need no request - their timers already meet sub-millisecond
/// deadlines - so this is inert and costs nothing at the shared call site.
#[cfg(not(target_os = "windows"))]
pub(crate) fn boost_timer_resolution() -> TimerResolutionGuard {
    TimerResolutionGuard::inert()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inert_guard_holds_no_period() {
        assert_eq!(TimerResolutionGuard::inert().period_ms, None);
    }

    /// Dropping an inert guard must not call `timeEndPeriod` for a period that
    /// was never begun - the `None` short-circuit is what guarantees it.
    #[test]
    fn dropping_an_inert_guard_is_a_no_op() {
        drop(TimerResolutionGuard::inert());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn non_windows_boost_stays_inert() {
        assert_eq!(boost_timer_resolution().period_ms, None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_boost_holds_the_millisecond_period() {
        // The suite never sets the kill switch, but honour it if a caller did.
        if std::env::var(DISABLE_ENV).as_deref() == Ok("1") {
            return;
        }
        let guard = boost_timer_resolution();
        assert_eq!(guard.period_ms, Some(TARGET_PERIOD_MS));
    }
}
