/// Useful type aliases that are used around the program.
pub mod aliases;

/// The module responsible for the CLI.
pub mod cli;

/// A module for helping with making predictions based on commit data.
pub mod prediction;

/// A module for the calendar structs.
pub mod calendar;

/// A module for misc methods for git. Note this isn't a git parsing library, just additional QoL methods.
pub mod git;

/// A module for misc utilities.
pub mod utils;

/// A module for application errors.
pub mod errors;
