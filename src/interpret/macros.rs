macro_rules! double_try {
    ($e:expr) => {
        match $e {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => return Ok(Err(e)),
            Err(e) => return Err(e),
        }
    };
}
pub(super) use double_try;
