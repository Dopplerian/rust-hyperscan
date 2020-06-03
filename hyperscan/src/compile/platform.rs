use std::mem::{self, MaybeUninit};

use anyhow::Result;
use bitflags::bitflags;
use foreign_types::{foreign_type, ForeignType};

use crate::errors::AsResult;
use crate::ffi;

/// Tuning Parameter
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tune {
    ///Generic
    Generic = ffi::HS_TUNE_FAMILY_GENERIC,

    /// Intel(R) microarchitecture code name Sandy Bridge
    SandyBridge = ffi::HS_TUNE_FAMILY_SNB,

    /// Intel(R) microarchitecture code name Ivy Bridge
    IvyBridge = ffi::HS_TUNE_FAMILY_IVB,

    /// Intel(R) microarchitecture code name Haswell
    Haswell = ffi::HS_TUNE_FAMILY_HSW,

    /// Intel(R) microarchitecture code name Silvermont
    Silvermont = ffi::HS_TUNE_FAMILY_SLM,

    /// Intel(R) microarchitecture code name Broadwell
    Broadwell = ffi::HS_TUNE_FAMILY_BDW,

    /// Intel(R) microarchitecture code name Skylake
    Skylake = ffi::HS_TUNE_FAMILY_SKL,

    /// Intel(R) microarchitecture code name Skylake Server
    SkylakeServer = ffi::HS_TUNE_FAMILY_SKX,

    /// Intel(R) microarchitecture code name Goldmont
    Goldmont = ffi::HS_TUNE_FAMILY_GLM,
}

bitflags! {
    /// CPU feature support flags
    #[derive(Default)]
    pub struct CpuFeatures: u64 {
        /// Intel(R) Advanced Vector Extensions 2 (Intel(R) AVX2)
        const AVX2 = ffi::HS_CPU_FEATURES_AVX2 as u64;
        /// Intel(R) Advanced Vector Extensions 512 (Intel(R) AVX512)
        const AVX512 = ffi::HS_CPU_FEATURES_AVX512 as u64;
    }
}

foreign_type! {
    /// A type containing information on the target platform
    /// which may optionally be provided to the compile calls
    pub unsafe type Platform {
        type CType = ffi::hs_platform_info_t;

        fn drop = free_platform_info;
    }
}

unsafe fn free_platform_info(p: *mut ffi::hs_platform_info_t) {
    mem::drop(Box::from_raw(p));
}

impl Platform {
    /// Test the current system architecture.
    pub fn is_valid() -> Result<()> {
        unsafe { ffi::hs_valid_platform().ok() }
    }

    /// Populates the platform information based on the current host.
    pub fn host() -> Result<Platform> {
        unsafe {
            let mut platform = MaybeUninit::zeroed();

            ffi::hs_populate_platform(platform.as_mut_ptr())
                .map(|_| Platform::from_ptr(Box::into_raw(Box::new(platform.assume_init()))))
        }
    }

    /// Constructs a target platform which may be used to guide the optimisation process of the compile.
    pub fn new(tune: Tune, cpu_features: CpuFeatures) -> Platform {
        unsafe {
            Platform::from_ptr(Box::into_raw(Box::new(ffi::hs_platform_info_t {
                tune: tune as u32,
                cpu_features: cpu_features.bits(),
                reserved1: 0,
                reserved2: 0,
            })))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_platform() {
        assert!(Platform::is_valid().is_ok())
    }
}
