mod crc16;
mod firmware;
mod profile;

pub use firmware::{Candidate, PatchOptions, ScanError, VerifyError, VerifyReport};
pub use profile::{Endianness, Profile, ProfileError};
