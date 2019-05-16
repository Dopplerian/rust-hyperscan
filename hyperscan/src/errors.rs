use core::fmt;

use failure::{AsFail, Error, Fail};

use crate::compile::Error as CompileError;
use crate::ffi;

/// Error Codes
#[derive(Debug, PartialEq, Fail)]
pub enum HsError {
    /// A parameter passed to this function was invalid.
    #[fail(display = "A parameter passed to this function was invalid.")]
    Invalid,

    /// A memory allocation failed.
    #[fail(display = "A memory allocation failed.")]
    NoMem,

    /// The engine was terminated by callback.
    #[fail(display = "The engine was terminated by callback.")]
    ScanTerminated,

    /// The pattern compiler failed with more detail.
    #[fail(display = "The pattern compiler failed with more detail, {}.", _0)]
    CompileError(CompileError),

    /// The given database was built for a different version of Hyperscan.
    #[fail(display = "The given database was built for a different version of Hyperscan.")]
    DbVersionError,

    /// The given database was built for a different platform (i.e., CPU type).
    #[fail(display = "The given database was built for a different platform (i.e., CPU type).")]
    DbPlatformError,

    /// The given database was built for a different mode of operation.
    #[fail(display = "The given database was built for a different mode of operation.")]
    DbModeError,

    /// A parameter passed to this function was not correctly aligned.
    #[fail(display = "A parameter passed to this function was not correctly aligned.")]
    BadAlign,

    /// The memory allocator did not correctly return memory suitably aligned.
    #[fail(display = "The memory allocator did not correctly return memory suitably aligned.")]
    BadAlloc,

    /// The scratch region was already in use.
    #[fail(display = "The scratch region was already in use.")]
    ScratchInUse,

    /// Unsupported CPU architecture.
    #[fail(display = "Unsupported CPU architecture.")]
    ArchError,

    /// Provided buffer was too small.
    #[fail(display = "Provided buffer was too small.")]
    InsufficientSpace,

    /// Unexpected internal error.
    #[fail(display = "Unexpected internal error.")]
    UnknownError,

    /// Unknown error code
    #[fail(display = "Unknown error code: {}", _0)]
    Code(ffi::hs_error_t),
}

impl From<ffi::hs_error_t> for HsError {
    fn from(err: ffi::hs_error_t) -> HsError {
        use HsError::*;

        match err {
            ffi::HS_INVALID => Invalid,
            ffi::HS_NOMEM => NoMem,
            ffi::HS_SCAN_TERMINATED => ScanTerminated,
            // ffi::HS_COMPILER_ERROR => HsError::CompileError,
            ffi::HS_DB_VERSION_ERROR => DbVersionError,
            ffi::HS_DB_PLATFORM_ERROR => DbPlatformError,
            ffi::HS_DB_MODE_ERROR => DbModeError,
            ffi::HS_BAD_ALIGN => BadAlign,
            ffi::HS_BAD_ALLOC => BadAlloc,
            ffi::HS_SCRATCH_IN_USE => ScratchInUse,
            ffi::HS_ARCH_ERROR => ArchError,
            ffi::HS_INSUFFICIENT_SPACE => InsufficientSpace,
            ffi::HS_UNKNOWN_ERROR => UnknownError,
            _ => Code(err),
        }
    }
}

pub trait AsResult
where
    Self: Sized,
{
    type Output;
    type Error: fmt::Debug + AsFail;

    fn ok(self) -> Result<Self::Output, Self::Error>;

    fn map<U, F: FnOnce(Self::Output) -> U>(self, op: F) -> Result<U, Self::Error> {
        self.ok().map(op)
    }

    fn and_then<U, F: FnOnce(Self::Output) -> Result<U, Self::Error>>(self, op: F) -> Result<U, Self::Error> {
        self.ok().and_then(op)
    }

    fn expect(self, msg: &str) -> Self::Output {
        self.ok().expect(msg)
    }
}

impl AsResult for ffi::hs_error_t {
    type Output = ();
    type Error = Error;

    fn ok(self) -> Result<Self::Output, Self::Error> {
        if self == ffi::HS_SUCCESS as ffi::hs_error_t {
            Ok(())
        } else {
            Err(HsError::from(self).into())
        }
    }
}
