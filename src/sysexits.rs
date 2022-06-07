//! A collection of partially standard exit codes from C's `sysexits.h`. The actual utility of this is likely very low. Descriptions were taken from the FreeBSD Library Functions Manual.
#![allow(dead_code)]

/// The command was used incorrectly, e.g., with the wrong number of arguments, a bad flag, a bad syntax in a parameter, or whatever.
pub const USAGE: i32 = 64;
/// The input data was incorrect in some way.  This should only be used for user's data and not system files.
pub const DATA_ERR: i32 = 65;
///  An input file (not a system file) did not exist orwas not readable. This could also include errors like "No message" to a mailer (if it cared to catch it).
pub const NO_INPUT: i32 = 66;
// TODO: Complete this
