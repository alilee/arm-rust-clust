// SPDX-License-Identifier: Unlicense

//! Uniform structure for errors and results.

/// Specified errors.
#[derive(Debug)]
pub enum Error {
    /// Function completes with no adverse conditions
    Success,
    /// Function failed with undefined error
    UnknownError,
}

/// Default error type for kernel functions.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_error() {
        error!("{:?}", Error::UnknownError);
    }

    #[test]
    fn return_error() {

        fn fn_ok() -> Result<f64> {
            Ok(1.0)
        }

        fn fn_error() -> Result<f64> {
            Err(Error::UnknownError)
        }

        fn fn_qmark() -> Result<f64> {
            fn_error()?;
            Ok(1.0)
        }

        assert_eq!(1.0, fn_ok().unwrap());
        assert_eq!(Err(Error::UnknownError), fn_error());
        assert_eq!(Err(Error::UnknownError), fn_qmark());
    }
}