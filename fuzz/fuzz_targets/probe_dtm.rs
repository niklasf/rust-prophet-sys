#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use prophet_sys::{
    prophet_tb_add_path, prophet_tb_create_decompress_ctx, prophet_tb_free_decompress_ctx,
    prophet_tb_init, prophet_tb_probe_dtm_dctx,
};
use std::ffi::c_int;
use std::sync::Once;

#[derive(Debug, Arbitrary)]
enum Piece {
    WhitePawn = 1,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

#[rustfmt::skip]
#[derive(Debug, Arbitrary)]
enum Square {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[derive(Debug, Arbitrary)]
enum Color {
    White = 0,
    Black,
}

#[derive(Debug, Arbitrary)]
struct Position {
    pieces: [Option<Piece>; 6],
    squares: [Square; 6],
    stm: Color,
    ep_square: Option<Square>,
}

fuzz_target!(|pos: Position| {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        prophet_tb_init();
        assert!(11 <= prophet_tb_add_path(c"tables".as_ptr()));
    });

    let pieces: [c_int; 6] = pos.pieces.map(|piece| piece.map_or(0, |p| p as c_int));
    let squares: [c_int; 6] = pos.squares.map(|square| square as c_int);

    unsafe {
        let dctx = prophet_tb_create_decompress_ctx();
        prophet_tb_probe_dtm_dctx(
            pieces.as_ptr(),
            squares.as_ptr(),
            pos.stm as c_int,
            pos.ep_square.map_or(0, |s| s as c_int),
            dctx,
        );
        prophet_tb_free_decompress_ctx(dctx);
    }
});
