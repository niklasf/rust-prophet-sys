#![no_main]

use std::{ffi::c_int, num::NonZeroU32, sync::Once};

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use prophet_sys::{
    prophet_tb_add_path, prophet_tb_create_decompress_ctx, prophet_tb_free_decompress_ctx,
    prophet_tb_init, prophet_tb_probe_dtm_dctx,
};
use shakmaty::{
    Bitboard, CastlingMode, Chess, Color, EnPassantMode, Piece, Position, Setup, Square,
};

#[derive(Debug, Arbitrary)]
struct EndgameSetup {
    pieces: [Option<(Square, Piece)>; 6],
    turn: Color,
    ep_square: Option<Square>,
}

impl EndgameSetup {
    fn to_setup(&self) -> Setup {
        Setup {
            board: self.pieces.into_iter().flatten().collect(),
            promoted: Bitboard::EMPTY,
            pockets: None,
            turn: self.turn,
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
        let mut pieces: [c_int; 6] = [0; 6];
        let mut squares: [c_int; 6] = [0; 6];
        for (i, (square, piece)) in pos.board().iter().enumerate() {
            pieces[i] = c_int::from(piece.role) + piece.color.fold_wb(0, 8);
            squares[i] = c_int::from(square);
        }

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
