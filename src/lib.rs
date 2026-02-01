#![allow(nonstandard_style)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe() {
        unsafe {
            prophet_tb_init();

            prophet_tb_deinit();
        }
    }
}
