//! An implementation of the
//! [Hybrid Logical Clock](http://www.cse.buffalo.edu/tech-reports/2014-04.pdf)
//! for Rust.

use time;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::sync::Mutex;

/// The `HLTimestamp` type stores a hybrid logical timestamp.
///
/// Such a timestamp is comprised of an "ordinary" seconds time and
/// a logical component. Timestamps are compared by seconds time first,
/// logical second.
///
/// # Examples
///
/// ```
/// use hlc::HLTimestamp;
/// let early = HLTimestamp::new(0, 0);
/// let middle = HLTimestamp::new(1, 0);
/// let late = HLTimestamp::new(1, 1);
/// assert!(early < middle && middle < late);
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HLTimestamp {
    seconds: i64,
    logical: u16,
}

impl HLTimestamp {
    /// Creates a new hybrid logical timestamp with the given seconds,
    /// nanoseconds, and logical ticks.
    ///
    /// # Examples
    ///
    /// ```
    /// use hlc::HLTimestamp;
    /// let ts = HLTimestamp::new(1, 2);
    /// assert_eq!(format!("{}", ts), "1+2");
    /// ```
    pub fn new(s: i64, l: u16) -> HLTimestamp {
        HLTimestamp {
            seconds: s,
            logical: l,
        }
    }
}

impl Display for HLTimestamp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{}+{}", self.seconds, self.logical))
    }
}

/// `State` is a hybrid logical clock.
///
/// # Examples
///
/// ```
/// use hlc::{HLTimestamp, State};
/// let mut s = State::new();
/// println!("{}", s.get_time()); // attach to outgoing event
/// let ext_event_ts = HLTimestamp::new(12345, 89); // external event's timestamp
/// let ext_event_recv_ts = s.update(ext_event_ts);
/// ```
///
/// If access to the clock isn't serializable, a convenience method returns
/// a `State` wrapped in a `Mutex`:
///
/// ```
/// use hlc::State;
/// let mut mu = State::new_sendable();
/// {
///     let mut s = mu.lock().unwrap();
///     s.get_time();
/// }
/// ```
pub struct State<F> {
    s: HLTimestamp,
    now: F,
}

impl State<()> {
    // Creates a standard hybrid logical clock, using `time::get_time` as
    // supplier of the physical clock's seconds time.
    pub fn new() -> State<fn() -> i64> {
        State::new_with(|| time::OffsetDateTime::now_utc().unix_timestamp())
    }

    // Returns the result of `State::new()`, wrapped in a `Mutex`.
    pub fn new_sendable() -> Mutex<State<fn() -> i64>> {
        Mutex::new(State::new())
    }
}

impl<F: FnMut() -> i64> State<F> {
    /// Creates a hybrid logical clock with the supplied seconds time. This is
    /// useful for tests or settings in which an alternative clock is used.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use hlc::{HLTimestamp, State};
    /// let mut times = vec![42, 43, 44];
    /// let mut s = State::new_with(move || times.pop().unwrap());
    /// let mut ts = s.get_time();
    /// assert_eq!(format!("{}", ts), "42+0");
    /// # }
    /// ```
    pub fn new_with(now: F) -> State<F> {
        State {
            s: HLTimestamp {
                seconds: 0,
                logical: 0,
            },
            now: now,
        }
    }

    /// Generates a timestamp from the clock.
    pub fn get_time(&mut self) -> HLTimestamp {
        let s = &mut self.s;
        let seconds = (self.now)();
        if s.seconds < seconds {
            s.seconds = seconds;
            s.logical = 0;
        } else {
            s.logical += 1;
        }
        s.clone()
    }

    /// Assigns a timestamp to an event which happened at the given timestamp
    /// on a remote system.
    pub fn update(&mut self, event: HLTimestamp) -> HLTimestamp {
        let (seconds, s) = ((self.now)(), &mut self.s);

        if seconds > event.seconds && seconds > s.seconds {
            s.seconds = seconds;
            s.logical = 0
        } else if event.seconds > s.seconds {
            s.seconds = event.seconds;
            s.logical = event.logical + 1;
        } else if s.seconds > event.seconds {
            s.logical += 1;
        } else {
            if event.logical > s.logical {
                s.logical = event.logical;
            }
            s.logical += 1;
        }
        s.clone()
    }
}

#[cfg(test)]
mod tests {
    extern crate time;
    use super::*;

    fn hlts(s: i64, l: u16) -> HLTimestamp {
        HLTimestamp::new(s, l)
    }

    #[test]
    fn it_works() {
        let zero = hlts(0, 0);
        let ops = vec![
            // Test cases in the form (seconds, event_ts, outcome).
            // Specifying event_ts as zero corresponds to calling `get_time`,
            // otherwise `update`.
            (2, zero, hlts(2, 0)),
            (2, zero, hlts(2, 1)),           // clock didn't move
            (1, zero, hlts(2, 2)),           // clock moved back
            (3, zero, hlts(3, 0)),           // finally ahead again
            (3, hlts(1, 3), hlts(3, 1)),     // event happens, seconds ahead but unchanged
            (3, hlts(3, 1), hlts(3, 2)),     // event happens at seconds, which is still unchanged
            (3, hlts(3, 99), hlts(3, 100)),  // event with larger logical, seconds unchanged
            (3, hlts(4, 100), hlts(4, 101)), // event with larger seconds, our seconds behind
            (5, hlts(4, 0), hlts(5, 0)),     // event behind seconds, but ahead of previous state
            (4, hlts(5, 99), hlts(5, 100)),
            (0, hlts(5, 50), hlts(5, 101)), // event at state, lower logical than state
        ];

        // Prepare fake clock and create State.
        let mut times = ops.iter().rev().map(|op| op.0).collect::<Vec<i64>>();
        let mut s = State::new_with(move || times.pop().unwrap());

        for op in &ops {
            let t = if op.1 == zero {
                s.get_time()
            } else {
                s.update(op.1.clone())
            };
            assert_eq!(t, op.2);
        }
    }
}
