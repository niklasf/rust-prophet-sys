#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use prophet_sys::{
    prophet_tb_add_path, prophet_tb_create_decompress_ctx, prophet_tb_free_decompress_ctx,
    prophet_tb_init, prophet_tb_probe_dtm_dctx,
};
use shakmaty::{
    Bitboard, CastlingMode, Chess, Color, EnPassantMode, Piece, Position, Setup, Square,
};
use std::ffi::c_int;
use std::iter::zip;
use std::num::NonZeroU32;
use std::sync::Once;

#[derive(Debug, Arbitrary)]
struct EndgameSetup {
    pieces: [Option<Piece>; 6],
    squares: [Square; 6],
    stm: Color,
    ep_square: Option<Square>,
}

impl EndgameSetup {
    fn to_setup(&self) -> Setup {
        Setup {
            board: zip(self.pieces, self.squares)
                .flat_map(|(piece, square)| piece.map(|piece| (square, piece)))
                .collect(),
            promoted: Bitboard::EMPTY,
            pockets: None,
            turn: self.stm,
            ep_square: self.ep_square,
            castling_rights: Bitboard::EMPTY,
            halfmoves: 0,
            fullmoves: NonZeroU32::MIN,
            remaining_checks: None,
        }
    }
}

fuzz_target!(|setup: EndgameSetup| {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        prophet_tb_init();
        assert!(11 <= prophet_tb_add_path(c"tables".as_ptr()));
    });

    if let Ok(pos) = setup.to_setup().position::<Chess>(CastlingMode::Chess960) {
        let pieces: [c_int; 6] = setup
            .pieces
            .map(|piece| piece.map_or(0, |p| c_int::from(p.role) + p.color.fold_wb(0, 8)));

        let squares: [c_int; 6] = setup.squares.map(c_int::from);

        unsafe {
            let dctx = prophet_tb_create_decompress_ctx();
            prophet_tb_probe_dtm_dctx(
                pieces.as_ptr(),
                squares.as_ptr(),
                pos.turn().fold_wb(0, 1),
                pos.ep_square(EnPassantMode::Always).map_or(64, c_int::from),
                dctx,
            );
            prophet_tb_free_decompress_ctx(dctx);
        }
    }
});
