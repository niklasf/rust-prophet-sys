#![no_main]

use arbitrary::Arbitrary;
use arrayvec::ArrayVec;
use prophet_sys::{
    prophet_tb_add_path, prophet_tb_init, prophet_tb_is_valid_position, prophet_tb_probe_dtm,
};
use shakmaty::{
    Bitboard, CastlingMode, Chess, Color, EnPassantMode, Piece, Position, Setup, Square,
};
use std::ffi::c_int;
use std::ffi::c_uchar;
use std::ffi::c_uint;
use std::iter::zip;
use std::num::NonZeroU32;
use std::sync::Once;

use libfuzzer_sys::fuzz_target;

#[derive(Debug, Arbitrary)]
struct EndgameSetup {
    pieces: [Option<(Square, Piece)>; 5],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dtm {
    Decisive(c_int),
    Draw,
}

unsafe fn probe_prophet(pos: &Chess) -> Dtm {
    let mut pieces: [c_int; 6] = [0; 6];
    let mut squares: [c_int; 6] = [0; 6];
    for ((square, piece), (p, s)) in zip(pos.board(), zip(&mut pieces, &mut squares)) {
        *p = c_int::from(piece.role) + piece.color.fold_wb(0, 8);
        *s = c_int::from(square);
    }
    let stm = pos.turn().fold_wb(0, 1);
    let ep_square = pos.ep_square(EnPassantMode::Legal).map_or(64, c_int::from);

    assert_eq!(
        unsafe { prophet_tb_is_valid_position(pieces.as_ptr(), squares.as_ptr(), stm, ep_square) },
        1
    );

    match unsafe { prophet_tb_probe_dtm(pieces.as_ptr(), squares.as_ptr(), stm, ep_square) } {
        -1001 => panic!("unknown prophet position"),
        1000 => Dtm::Draw,
        dtm => Dtm::Decisive(dtm),
    }
}

unsafe fn probe_gaviota(pos: &Chess) -> Dtm {
    let mut ws = ArrayVec::<c_uint, 6>::new();
    let mut bs = ArrayVec::<c_uint, 6>::new();
    let mut wp = ArrayVec::<c_uchar, 6>::new();
    let mut bp = ArrayVec::<c_uchar, 6>::new();
    for (sq, piece) in pos.board() {
        piece.color.fold_wb(&mut ws, &mut bs).push(c_uint::from(sq));
        piece
            .color
            .fold_wb(&mut wp, &mut bp)
            .push(c_uchar::from(piece.role));
    }
    ws.push(gaviota_sys::TB_squares::tb_NOSQUARE as c_uint);
    bs.push(gaviota_sys::TB_squares::tb_NOSQUARE as c_uint);
    wp.push(gaviota_sys::TB_pieces::tb_NOPIECE as c_uchar);
    bp.push(gaviota_sys::TB_pieces::tb_NOPIECE as c_uchar);

    let mut info: c_uint = 0;
    let mut plies: c_uint = 0;

    let result = unsafe {
        gaviota_sys::tb_probe_hard(
            pos.turn().fold_wb(
                gaviota_sys::TB_sides::tb_WHITE_TO_MOVE,
                gaviota_sys::TB_sides::tb_BLACK_TO_MOVE,
            ) as c_uint,
            pos.ep_square(EnPassantMode::Legal)
                .map_or(gaviota_sys::TB_squares::tb_NOSQUARE as c_uint, c_uint::from),
            gaviota_sys::TB_castling::tb_NOCASTLE.0,
            ws.as_ptr(),
            bs.as_ptr(),
            wp.as_ptr(),
            bp.as_ptr(),
            &mut info,
            &mut plies,
        )
    };

    let plies = plies as c_int;

    match gaviota_sys::TB_return_values(info) {
        gaviota_sys::TB_return_values::tb_FORBID => panic!("unknown gaviota position"),
        _ if result == 0 => panic!("gaviota probe failed with result {result} and info {info}"),
        gaviota_sys::TB_return_values::tb_DRAW => Dtm::Draw,
        gaviota_sys::TB_return_values::tb_WMATE => Dtm::Decisive(pos.turn().fold_wb(plies, -plies)),
        gaviota_sys::TB_return_values::tb_BMATE => Dtm::Decisive(pos.turn().fold_wb(-plies, plies)),
        _ => panic!("unknown gaviota info {info}"),
    }
}

fuzz_target!(|setup: EndgameSetup| {
    static INIT_TB: Once = Once::new();
    INIT_TB.call_once(|| unsafe {
        // Prophet
        prophet_tb_init();
        assert!(145 <= prophet_tb_add_path(c"tables".as_ptr()));

        // Gaviota
        assert_ne!(gaviota_sys::tbcache_init(1024 * 1024, 50), 0, "tbache_init");

        let mut paths = gaviota_sys::tbpaths_init();
        assert!(!paths.is_null(), "tbpaths_init");
        paths = gaviota_sys::tbpaths_add(paths, c"gaviota".as_ptr());
        assert!(!paths.is_null(), "tbpaths_add");
        assert!(
            !gaviota_sys::tb_init(
                1,
                gaviota_sys::TB_compression_scheme::tb_CP4 as c_int,
                paths
            )
            .is_null(),
            "tb_init"
        );
    });

    if let Ok(pos) = setup.to_setup().position::<Chess>(CastlingMode::Chess960) {
        unsafe {
            assert_eq!(probe_gaviota(&pos), probe_prophet(&pos));
        }
    }
});
