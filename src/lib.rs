#![allow(nonstandard_style)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::ffi::c_int;

    use super::*;

    #[test]
    fn test_probe() {
        unsafe {
            prophet_tb_init();

            let dctx = prophet_tb_create_decompress_ctx();

            // 4k3/8/8/8/8/8/4Q3/4K3 b - - 0 1
            let pieces: [c_int; 6] = [
                6,  // white king
                5,  // white queen
                14, // black king
                0,  // no piece
                0,  // no piece
                0,  // no piece
            ];
            let squares: [c_int; 6] = [
                4,  // e1
                12, // e2
                60, // e8
                0,  // unused
                0,  // unused
                0,  // unused
            ];
            let dtm = prophet_tb_probe_dtm_dctx(
                pieces.as_ptr(),
                squares.as_ptr(),
                1,  // black to move
                64, // no en-passant square
                dctx,
            );
            assert_eq!(dtm, -1001); // no tables added yet

            assert!(11 <= prophet_tb_add_path(c"tables".as_ptr()));
            let dtm = prophet_tb_probe_dtm_dctx(pieces.as_ptr(), squares.as_ptr(), 1, 64, dctx);
            assert_eq!(dtm, 16);

            prophet_tb_free_decompress_ctx(dctx);

            prophet_tb_deinit();
        }
    }
}
