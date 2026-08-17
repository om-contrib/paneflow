//! Release-capable probe for the terminal coalescing window.
//!
//! [`view::probe_enabled`](super::view::probe_enabled) (`PANEFLOW_LATENCY_PROBE`)
//! is `#[cfg(debug_assertions)]`, but frame pacing has to be measured in the
//! shipped release build - the platform timer, not the optimizer, is what this
//! questions.
//!
//! `TerminalView` opens a 4 ms window to coalesce backend wakeups before it
//! paints. Windows services that wait on the system timer, whose default tick
//! is 15.625 ms, so the window can stay open several times longer than its
//! nominal duration - and the *spread*, not the mean, is what reads as a
//! stutter while typing, because it lands each keystroke echo in a different
//! frame. This probe reports the distribution actually observed at runtime.

use std::sync::OnceLock;
use std::time::Duration;

use parking_lot::Mutex;

/// Set to `1` to emit the summary. Off by default: the probe costs an
/// `Instant::now()` pair plus a push per closed window.
const ENABLE_ENV: &str = "PANEFLOW_FRAME_PROBE";

/// Windows covered per summary line. Large enough for the percentiles to
/// settle, small enough that a few seconds of typing produces one.
const SAMPLE_BATCH: usize = 200;

/// Nominal window duration requested in `view.rs`, echoed in the summary so the
/// gap between asked-for and observed is self-evident in the log.
const NOMINAL_MS: f64 = 4.0;

struct ProbeState {
    /// Durations (µs) of windows that stayed open until the timer fired.
    timer_fired_us: Vec<u32>,
    /// Windows closed early by the dequeue cap. Excluded from the percentiles
    /// because they never waited on the timer, but counted so a summary built
    /// from a busy burst is not mistaken for an idle one.
    closed_early: usize,
}

impl ProbeState {
    const fn new() -> Self {
        Self {
            timer_fired_us: Vec::new(),
            closed_early: 0,
        }
    }
}

static STATE: Mutex<ProbeState> = Mutex::new(ProbeState::new());

/// Whether the probe is armed. Read once - the env var cannot change under a
/// running process, and this is called on every batch window.
pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var(ENABLE_ENV).as_deref() == Ok("1"))
}

/// Record one closed coalescing window. `timer_fired` separates a genuine timer
/// expiry from an early close on the event cap.
pub(crate) fn record_batch_window(elapsed: Duration, timer_fired: bool) {
    if !enabled() {
        return;
    }

    let mut state = STATE.lock();
    if !timer_fired {
        state.closed_early = state.closed_early.saturating_add(1);
        return;
    }

    state
        .timer_fired_us
        .push(u32::try_from(elapsed.as_micros()).unwrap_or(u32::MAX));
    if state.timer_fired_us.len() < SAMPLE_BATCH {
        return;
    }

    let mut samples = std::mem::take(&mut state.timer_fired_us);
    let closed_early = std::mem::replace(&mut state.closed_early, 0);
    // Release before formatting + logging: the render path takes this lock on
    // every window and must not queue behind a log write.
    drop(state);

    log::info!(
        target: "paneflow::terminal::frame_probe",
        "{}",
        summarize(&mut samples, closed_early)
    );
}

fn summarize(samples: &mut [u32], closed_early: usize) -> String {
    samples.sort_unstable();
    let count = samples.len();
    let ms = |micros: u32| f64::from(micros) / 1_000.;
    let mean_ms =
        samples.iter().map(|&us| f64::from(us)).sum::<f64>() / count.max(1) as f64 / 1_000.;

    format!(
        "coalescing window over {count} timer closures: p50={:.1}ms p95={:.1}ms max={:.1}ms \
         mean={mean_ms:.1}ms (nominal {NOMINAL_MS:.0}ms), {closed_early} closed early on the event cap",
        ms(percentile(samples, 0.50)),
        ms(percentile(samples, 0.95)),
        ms(samples.last().copied().unwrap_or_default()),
    )
}

/// Nearest-rank percentile over an already-sorted slice.
fn percentile(sorted: &[u32], fraction: f64) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let last = sorted.len() - 1;
    let index = (last as f64 * fraction).round() as usize;
    sorted[index.min(last)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_spans_the_whole_range() {
        let sorted: Vec<u32> = (0..=100).collect();
        assert_eq!(percentile(&sorted, 0.0), 0);
        assert_eq!(percentile(&sorted, 0.5), 50);
        assert_eq!(percentile(&sorted, 0.95), 95);
        assert_eq!(percentile(&sorted, 1.0), 100);
    }

    #[test]
    fn percentile_of_empty_slice_is_zero() {
        assert_eq!(percentile(&[], 0.5), 0);
    }

    #[test]
    fn percentile_of_single_sample_is_that_sample() {
        assert_eq!(percentile(&[42], 0.0), 42);
        assert_eq!(percentile(&[42], 0.95), 42);
    }

    /// The summary must surface the gap between the requested 4 ms and a real
    /// Windows tick - that contrast is the entire point of the line.
    #[test]
    fn summary_reports_percentiles_against_the_nominal_window() {
        let mut samples: Vec<u32> = (0..200).map(|_| 15_600).collect();
        samples[199] = 31_200;

        let summary = summarize(&mut samples, 7);

        assert!(summary.contains("200 timer closures"), "{summary}");
        assert!(summary.contains("p50=15.6ms"), "{summary}");
        assert!(summary.contains("max=31.2ms"), "{summary}");
        assert!(summary.contains("nominal 4ms"), "{summary}");
        assert!(summary.contains("7 closed early"), "{summary}");
    }

    /// `record_batch_window` is a no-op unless armed, so the shipped default
    /// costs nothing beyond the `enabled()` load.
    #[test]
    fn recording_while_disarmed_leaves_no_state() {
        if enabled() {
            return; // The suite was launched with the probe armed.
        }
        record_batch_window(Duration::from_millis(15), true);
        let state = STATE.lock();
        assert!(state.timer_fired_us.is_empty());
        assert_eq!(state.closed_early, 0);
    }
}
