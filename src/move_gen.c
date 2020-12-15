#include "move_gen.h"
#include "found_magics.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

/**
 * knight and king move generation lookup table.
 * for each square on the board, the lookup table contains a bitboard with bits set for each valid move for a knight/king on that square */
static bitboard move_gen_knights[64];
static bitboard move_gen_kings[64];

/**
 * pre-generate the knight lookup table */
static void move_gen_init_knights() {
  for(board_pos pos = 0; pos < 64; pos++) {
    int x, y;
    board_pos_to_xy(pos, &x, &y);
    bitboard moves = 0;
    for(int i = -1; i <= 1; i += 2) {
      for(int j = -1; j <= 1; j += 2) {
        // generate move offset by (1,2) and (2,1) and add to moves if square is in bounds
        board_pos s1 = board_pos_from_xy(x + i*1, y + j*2);
        board_pos s2 = board_pos_from_xy(x + i*2, y + j*1);

        if(s1 != BOARD_POS_INVALID) {
          moves = bitboard_set_square(moves, s1);
        }
        if(s2 != BOARD_POS_INVALID) {
          moves = bitboard_set_square(moves, s2);
        }
      }
      move_gen_knights[pos] = moves;
    }
  }
}

/**
 * pre-generate the king lookup table */
static void move_gen_init_kings() {
  for(board_pos pos = 0; pos < 64; pos++) {
    int x, y;
    board_pos_to_xy(pos, &x, &y);
    bitboard moves = 0;
    for(int i = -1; i <= 1; i += 1) {
      for(int j = -1; j <= 1; j += 1) {
        if(i == 0 && j == 0)
          continue;

        board_pos square = board_pos_from_xy(x + i, y + j);
        if(square != BOARD_POS_INVALID) {
          moves = bitboard_set_square(moves, square);
        }
      }
      move_gen_kings[pos] = moves;
    }
  }
}

/**
 * given a occupancy bitboard, return valid sliding moves from (x,y) in (dx,dy) direction */
static bitboard gen_ray_moves(bitboard occupancy, int x, int y, int dx, int dy) {
  bitboard moves = 0;
  x = x + dx;
  y = y + dy;
  while(x <= 7 && y <= 7 && x >= 0 && y >= 0) {
    moves = bitboard_set_square(moves, board_pos_from_xy(x, y));
    if(bitboard_check_square(occupancy, board_pos_from_xy(x, y))) {
      break;
    }
    x += dx;
    y += dy;
  }

  return moves;
}

/**
 * given a occupancy bitboard and a bishop/rook square, generate valid moves
 * this is used to pre-generate these moves. see fast lookups below */
static bitboard gen_rook_moves(bitboard occupancy, board_pos square) {
  int x, y;
  board_pos_to_xy(square, &x, &y);
  bitboard moves = 0;
  moves |= gen_ray_moves(occupancy, x, y, 1, 0);
  moves |= gen_ray_moves(occupancy, x, y, -1, 0);
  moves |= gen_ray_moves(occupancy, x, y, 0, 1);
  moves |= gen_ray_moves(occupancy, x, y, 0, -1);
  return moves;
}

static bitboard gen_bishop_moves(bitboard occupancy, board_pos square) {
  int x, y;
  board_pos_to_xy(square, &x, &y);
  bitboard moves = 0;
  moves |= gen_ray_moves(occupancy, x, y, 1, 1);
  moves |= gen_ray_moves(occupancy, x, y, -1, 1);
  moves |= gen_ray_moves(occupancy, x, y, 1, -1);
  moves |= gen_ray_moves(occupancy, x, y, -1, -1);
  return moves;
}

/**
 * sliding pieces (bishop and rook) move lookup table */
#define SLIDING_TABLE_SIZE 107648 // sum of 1 << each entry in rook_magic_bits and bishop_magic_bits
static bitboard sliding_magic_moves[SLIDING_TABLE_SIZE];
// starting indexes for each position in the table
static bitboard* rook_magic_table_ptr[64];
static bitboard* bishop_magic_table_ptr[64];

/**
 * given an occupancy bitboard + square with rook/bishop, generate the index within that square's section of the sliding table */
static uint64_t magic_index_rook(bitboard occupancy, board_pos square) {
  return ((occupancy & rook_magic_masks[square]) * rook_magic_factors[square]) >> (64 - rook_magic_bits[square]);
}

static uint64_t magic_index_bishop(bitboard occupancy, board_pos square) {
  return ((occupancy & bishop_magic_masks[square]) * bishop_magic_factors[square]) >> (64 - bishop_magic_bits[square]);
}

/**
 * given an occupancy bitboard, lookup valid moves for a rook/bishop at square
 * because occupancy doesn't distinguish between the players, the result includes capturing all occupied squares
 * to get only valid attacks against the opponent, do (result & ~us_occupancy) */
static bitboard rook_move_lookup(bitboard occupancy, board_pos square) {
  assert(square < 64);
  return rook_magic_table_ptr[square][magic_index_rook(occupancy, square)];
}

static bitboard bishop_move_lookup(bitboard occupancy, board_pos square) {
  assert(square < 64);
  return bishop_magic_table_ptr[square][magic_index_bishop(occupancy, square)];
}

/**
 * lookup valid moves for a queen (just rook | bishop) */
static bitboard queen_magic_lookup(bitboard occupancy, board_pos square) {
  return rook_move_lookup(occupancy, square) | bishop_move_lookup(occupancy, square);
}

/**
 * given a bitboard mask, generate all permutations of setting/clearing bits in current that are set in maks
 * call func on each permutation */
static void permute_mask(bitboard mask, bitboard current, void (*func)(bitboard, board_pos, int), board_pos func_arg0, int func_arg1) {
  if(!mask) {
    func(current, func_arg0, func_arg1);
  } else {
    // find first set bit in mask, clear it, and generate permutations on current with that bit set + cleared
    board_pos set_i = bitboard_scan_lsb(mask);
    bitboard new_mask = bitboard_clear_square(mask, set_i);
    permute_mask(new_mask, bitboard_set_square(current, set_i), func, func_arg0, func_arg1);
    permute_mask(new_mask, bitboard_clear_square(current, set_i), func, func_arg0, func_arg1);
  }
}

/**
 * callbacks to be passed to permute_mask
 * for this board occupancy, generate rook/bishop moves and put in sliding_magic_moves lookup table */
static void move_gen_init_sliders_rook(bitboard occupancy, board_pos pos, int pos_tbl_start) {
  bitboard moves = gen_rook_moves(occupancy, pos);
  uint64_t index = pos_tbl_start + magic_index_rook(occupancy, pos);
  assert(index < SLIDING_TABLE_SIZE);
  assert(sliding_magic_moves[index] == 0xffffffffffffffffULL || sliding_magic_moves[index] == moves);
  sliding_magic_moves[index] = moves;
}

static void move_gen_init_sliders_bishop(bitboard occupancy, board_pos pos, int pos_tbl_start) {
  bitboard moves = gen_bishop_moves(occupancy, pos);
  uint64_t index = pos_tbl_start + magic_index_bishop(occupancy, pos);
  assert(index < SLIDING_TABLE_SIZE);
  assert(sliding_magic_moves[index] == 0xffffffffffffffffULL || sliding_magic_moves[index] == moves);
  sliding_magic_moves[index] = moves;
}

/**
 * pre-generate the sliding_magic_moves array */
static void move_gen_init_sliders() {
  memset(sliding_magic_moves, 0xff, sizeof(sliding_magic_moves));
  int tbl_index = 0;
  for(board_pos pos = 0; pos < 64; pos++) {
    rook_magic_table_ptr[pos] = sliding_magic_moves + tbl_index;
    permute_mask(rook_magic_masks[pos], 0, &move_gen_init_sliders_rook, pos, tbl_index);
    tbl_index += (1 << rook_magic_bits[pos]);
  }
  for(board_pos pos = 0; pos < 64; pos++) {
    bishop_magic_table_ptr[pos] = sliding_magic_moves + tbl_index;
    permute_mask(bishop_magic_masks[pos], 0, &move_gen_init_sliders_bishop, pos, tbl_index);
    tbl_index += (1 << bishop_magic_bits[pos]);
  }
  assert(tbl_index == SLIDING_TABLE_SIZE);
}

// indexed [player][double_rank_ahead][rank_ahead][square]
static bitboard move_gen_pawns[2][2][8][64];

/**
 * pre-generate move_gen_pawns */
static void move_gen_init_pawns() {
  for(int player = 0; player < 2; player++) {
    for (board_pos pos = 0; pos < 64; pos++) {
      int x, y;
      board_pos_to_xy(pos, &x, &y);
      if(y == 0 || y == 7) {
        continue;
      }
      for(unsigned int ahead = 0; ahead < 8; ahead++) {
        for(unsigned int double_ahead = 0; double_ahead < 2; double_ahead++) {
          bitboard moves = 0;
          int dir = player == WHITE ? 1 : -1;
          // check move directly ahead
          if(!(ahead & 2U)) {
            moves = bitboard_set_square(moves, board_pos_from_xy(x, y + dir));
            // check move two ahead
            if(!double_ahead && ((player == WHITE && y == 1) || (player == BLACK && y == 6))) {
              moves = bitboard_set_square(moves, board_pos_from_xy(x, y + 2*dir));
            }
          }
          // check captures
          if(x >= 1 && (ahead & 1U)) {
            moves = bitboard_set_square(moves, board_pos_from_xy(x - 1, y + dir));
          }
          if(x <= 6 && (ahead & 4U)) {
            moves = bitboard_set_square(moves, board_pos_from_xy(x + 1, y + dir));
          }
          // set lookup table
          move_gen_pawns[player][double_ahead][ahead][pos] = moves;
        }
      }
    }
  }
}

/**
 * given an occupancy bitboard, lookup valid moves for a pawn on square with color player */
static bitboard pawn_move_lookup(bitboard occupancy, board_pos square, int player) {
  int dir = player == WHITE ? 1 : -1;
  // get the three squares forward of the pawn, and the square two squares ahead
  uint8_t forward_rank = (occupancy >> (square - 1 + dir * 8)) & 0x07;
  uint8_t double_forward_rank = (occupancy >> (square + dir * 16)) & 0x01;

  return move_gen_pawns[player][double_forward_rank][forward_rank][square];
}

/**
 * generate moves for the piece at the given square on board
 * this is just regular moves -- does not include castles */
static bitboard move_gen_reg_moves_mask(bitboard occupancy_for_sliders, bitboard occupancy_for_pawns, int piece, int player, board_pos square) {
  switch(piece) {
    case KING:
      return move_gen_kings[square];
    case KNIGHT:
      return move_gen_knights[square];
    case PAWN:
      return pawn_move_lookup(occupancy_for_pawns, square, player);
    case ROOK:
      return rook_move_lookup(occupancy_for_sliders, square);
    case BISHOP:
      return bishop_move_lookup(occupancy_for_sliders, square);
    case QUEEN:
      return queen_magic_lookup(occupancy_for_sliders, square);
    default:
      assert(0);
  }
}

/**
 * generate the occupancy mask for slider/pawn lookups */
static bitboard board_occupancy_for_sliders_lookups(const board *board) {
  return board->players[WHITE] | board->players[BLACK];
}
static bitboard board_occupancy_for_pawns_lookups(const board *board) {
  bitboard occupancy = board->players[WHITE] | board->players[BLACK];
  if(board->flags & BOARD_FLAGS_EP_PRESENT) {
    occupancy = bitboard_set_square(occupancy, board->flags & BOARD_FLAGS_EP_SQUARE);
  }
  return occupancy;
}

#define MOVE_GEN_MODE_NORMAL 0
#define MOVE_GEN_MODE_CASTLES 2

void move_gen_init(move_gen *move_gen, board *board) {
  int player = board_player_to_move(board);
  move_gen->board = board;
  move_gen->occupancy_for_sliders = board_occupancy_for_sliders_lookups(board);
  move_gen->occupancy_for_pawns = board_occupancy_for_pawns_lookups(board);
  move_gen->final_moves_mask = ~board->players[player];
  move_gen->cur_mode = MOVE_GEN_MODE_NORMAL;
  move_gen->cur_square = 0;
  move_gen->cur_moves = 0;
  move_gen->cur_promotion = 0;
  move_gen->cur_piece_type = 0;
}

#define MOVE_FLAGS_PREV_FLAGS     0x0000000ffffULL
#define MOVE_FLAGS_SRC            0x000003f0000ULL
#define MOVE_SHIFT_SRC            16
#define MOVE_FLAGS_DST            0x0000fc00000ULL
#define MOVE_SHIFT_DST            22
#define MOVE_FLAGS_IS_PROMOTE     0x00010000000ULL
#define MOVE_SHIFT_IS_PROMOTE     28
#define MOVE_FLAGS_PROMOTE_PIECE  0x000e0000000ULL
#define MOVE_SHIFT_PROMOTE_PIECE  29
#define MOVE_FLAGS_IS_CAPTURE     0x00100000000ULL
#define MOVE_SHIFT_IS_CAPTURE     32
#define MOVE_FLAGS_CAPTURE_PIECE  0x00e00000000ULL
#define MOVE_SHIFT_CAPTURE_PIECE  33
#define MOVE_FLAGS_CAPTURE_SQUARE 0x3f000000000ULL
#define MOVE_SHIFT_CAPTURE_SQUARE 36

board_pos move_source_square(move move) {
  return (move & MOVE_FLAGS_SRC) >> MOVE_SHIFT_SRC;
}

board_pos move_destination_square(move move) {
  return (move & MOVE_FLAGS_DST) >> MOVE_SHIFT_DST;
}

int move_is_promotion(move move) {
  return move & MOVE_FLAGS_IS_PROMOTE ? 1:0;
}

int move_promotion_piece(move move) {
  if(!move_is_promotion(move))
    return -1;
  else
    return (move & MOVE_FLAGS_PROMOTE_PIECE) >> MOVE_SHIFT_PROMOTE_PIECE;
}

int move_is_capture(move move) {
  return move & MOVE_FLAGS_IS_CAPTURE ? 1:0;
}

int move_capture_piece(move move) {
  if(!move_is_capture(move))
    return -1;
  else
    return (move & MOVE_FLAGS_CAPTURE_PIECE) >> MOVE_SHIFT_CAPTURE_PIECE;
}

board_pos move_capture_square(move move) {
  if(!move_is_capture(move))
    return BOARD_POS_INVALID;
  else
    return  (move & MOVE_FLAGS_CAPTURE_SQUARE) >> MOVE_SHIFT_CAPTURE_SQUARE;
}

/**
 * create a move from the given components */
static move construct_move(uint16_t board_flags, board_pos src, board_pos dst, int is_promotion, int promote_piece, int is_capture, int capture_piece, board_pos capture_pos) {
  return
    ((uint64_t)board_flags & MOVE_FLAGS_PREV_FLAGS) +
    (((uint64_t)src << MOVE_SHIFT_SRC) & MOVE_FLAGS_SRC) +
    (((uint64_t)dst << MOVE_SHIFT_DST) & MOVE_FLAGS_DST) +
    ((uint64_t)!!is_promotion << MOVE_SHIFT_IS_PROMOTE) +
    (((uint64_t)promote_piece << MOVE_SHIFT_PROMOTE_PIECE) & MOVE_FLAGS_PROMOTE_PIECE) +
    ((uint64_t)!!is_capture << MOVE_SHIFT_IS_CAPTURE) +
    (((uint64_t)capture_piece << MOVE_SHIFT_CAPTURE_PIECE) & MOVE_FLAGS_CAPTURE_PIECE) +
    (((uint64_t)capture_pos << MOVE_SHIFT_CAPTURE_SQUARE) & MOVE_FLAGS_CAPTURE_SQUARE);
}

/**
 * given an en passant target square, get the pawn square to which it cooresponds */
static board_pos en_passant_target_to_pawn_pos(board_pos ep_target) {
  int x, y;
  board_pos_to_xy(ep_target, &x, &y);
  // en passant must be on rank 3 or 6
  if(y == 2) {
    return board_pos_from_xy(x, y + 1);
  } else if(y == 5) {
    return board_pos_from_xy(x, y - 1);
  } else {
    assert(0);
  }
}

static char promote_codes[6] = {'k', 'p', 'n', 'r', 'b', 'q'};

void move_to_str(move move, char *res_str) {
  board_pos_to_str(move_source_square(move), res_str);
  board_pos_to_str(move_destination_square(move), res_str + 2);
  if(move_is_promotion(move)) {
    res_str[4] = promote_codes[move_promotion_piece(move)];
    res_str[5] = '\0';
  } else {
    res_str[4] = '\0';
  }
}

move move_from_str(char *move_str, const board *board) {
  // seperate src, dst, and promotion parts of string
  char src_str[3];
  memcpy(src_str, move_str, 2 * sizeof(char));
  src_str[2] = '\0';
  char dst_str[3];
  memcpy(dst_str, move_str + 2, 2 * sizeof(char));
  dst_str[2] = '\0';
  // handle promotion char if present
  int is_promote = 0;
  char promote_char;
  int promote_piece = -1;
  if(move_str[4] != '\0') {
    is_promote = 1;
    promote_char = move_str[4];
    for(int p = 0; p < 6; p++) {
      if(promote_codes[p] == promote_char)
        promote_piece = p;
    }
    assert(promote_piece != -1);
  }
  // parse src + dst
  board_pos src = board_pos_from_str(src_str);
  board_pos dst = board_pos_from_str(dst_str);
  // check for capturing
  int is_capture = 0;
  board_pos capture_pos = BOARD_POS_INVALID;
  int capture_piece = board_piece_on_square(board, dst);
  if(capture_piece != -1) {
    assert(board_player_on_square(board, dst) == !board_player_to_move(board));
    is_capture = 1;
    capture_pos = dst;
  }
  // check for en passant
  if(dst == board_get_en_passant_target(board) && board_piece_on_square(board, src) == PAWN) {
    is_capture = 1;
    capture_pos = en_passant_target_to_pawn_pos(board_get_en_passant_target(board));
    capture_piece = board_piece_on_square(board, capture_pos);
    assert(capture_piece == PAWN);
  }

  return construct_move(board->flags, src, dst, is_promote, promote_piece, is_capture, capture_piece, capture_pos);
}

bitboard board_is_square_attacked(const board *board, board_pos square, int attacking_player) {
  assert(attacking_player == WHITE || attacking_player == BLACK);
  int defending_player = !attacking_player;
  // hits on attacking pieces
  bitboard attack_hits = 0;
  bitboard attackers_mask = board->players[attacking_player];
  bitboard occ_slide = board_occupancy_for_sliders_lookups(board);
  bitboard occ_pawn = board_occupancy_for_pawns_lookups(board);
  // in order to find attacks, treat the square as a piece of each type and check the intersection of legal moves and the opponent
  for(int piece = KING; piece <= KNIGHT; piece++) {
    assert(piece == KING || piece == PAWN || piece == KNIGHT);
    attack_hits |= move_gen_reg_moves_mask(occ_slide, occ_pawn, piece,
                                           defending_player, square) & (board->pieces[piece]);
  }
  for(int piece = ROOK; piece <= BISHOP; piece++) {
    assert(piece == ROOK || piece == BISHOP);
    attack_hits |= move_gen_reg_moves_mask(occ_slide, occ_pawn, piece,
                                           defending_player, square) & (board->pieces[piece] | board->pieces[QUEEN]);
  }
  attack_hits &= attackers_mask;
  return attack_hits;
}

void move_gen_make_move(board *board, move move) {
  board_invariants(board);
  assert(board->flags == (move & MOVE_FLAGS_PREV_FLAGS));
  board_pos src = move_source_square(move);
  board_pos dst = move_destination_square(move);
  int piece = board_piece_on_square(board, src);
  int player = board_player_to_move(board);
  int opponent = !player;

  assert(piece != -1);
  assert(!bitboard_check_square(board->players[opponent], dst) || move_is_capture(move));
  // if move is capture, clear dst for opponent
  if(move_is_capture(move)) {
    board_pos cap_square = move_capture_square(move);
    // en passant capture
    board_pos ep_target = board_get_en_passant_target(board);
    if(ep_target != BOARD_POS_INVALID && ep_target  == dst) {
      cap_square = en_passant_target_to_pawn_pos(ep_target);
    }
    int cap_piece = board_piece_on_square(board, cap_square);
    assert(cap_piece != -1);
    assert(cap_square != src);
    assert(board_player_on_square(board, cap_square) != player);
    board->players[opponent] = bitboard_clear_square(board->players[opponent], cap_square);
    board->pieces[cap_piece] = bitboard_clear_square(board->pieces[cap_piece], cap_square);
  }
  // move piece from src to dst and clear src
  board->pieces[piece] = bitboard_set_square(board->pieces[piece], dst);
  board->players[player] = bitboard_set_square(board->players[player], dst);
  board->pieces[piece] = bitboard_clear_square(board->pieces[piece], src);
  board->players[player] = bitboard_clear_square(board->players[player], src);
  // clear en passant target
  board->flags &= ~(BOARD_FLAGS_EP_PRESENT);
  // set en passant target if move is double pawn push
  if(piece == PAWN && ((src - dst) == 16 || (dst - src) == 16)) {
    #ifndef NDEBUG
      int x, y;
      board_pos_to_xy(src, &x, &y);
      assert((board_player_to_move(board) == WHITE && y == 1) || (board_player_to_move(board) == BLACK && y == 6));
    #endif
    // generate en passant target (one behind square)
    board_pos ep_target;
    if(dst > src) {
      ep_target = src + 8;
    } else {
      ep_target = src - 8;
    }
    board->flags |= BOARD_FLAGS_EP_PRESENT;
    board->flags &= ~BOARD_FLAGS_EP_SQUARE;
    board->flags |= (ep_target & BOARD_FLAGS_EP_SQUARE);
  }
  // TODO: revoke castling rights if needed + castling + promotion
  // flip player to move
  board->flags ^= BOARD_FLAGS_TURN;
  board_invariants(board);
}

void move_gen_unmake_move(board *board, move move) {
  board_invariants(board);
  // restore flags
  board->flags = move & MOVE_FLAGS_PREV_FLAGS;
  // extract info from moves
  board_pos src = move_source_square(move);
  board_pos dst = move_destination_square(move);
  int piece_maybe_promoted = board_piece_on_square(board, dst);
  int piece = move_is_promotion(move) ? PAWN : piece_maybe_promoted;
  assert(piece != -1);
  // player is the player that made the move
  int player = board_player_to_move(board);
  int opponent = !player;
  // move dst to src
  board->pieces[piece] = bitboard_clear_square(board->pieces[piece], dst);
  board->players[player] = bitboard_clear_square(board->players[player], dst);
  board->pieces[piece] = bitboard_set_square(board->pieces[piece], src);
  board->players[player] = bitboard_set_square(board->players[player], src);
  // restore captures
  if(move_is_capture(move)) {
    int cap_piece = move_capture_piece(move);
    int cap_square = move_capture_square(move);
    board->pieces[cap_piece] = bitboard_set_square(board->pieces[cap_piece], cap_square);
    board->players[opponent] = bitboard_set_square(board->players[opponent], cap_square);
  }
  // TODO: handle castling
  board_invariants(board);
}

static move move_gen_next_from_cur_moves(move_gen *generator) {
  assert(generator->cur_moves);
  board_pos dst = bitboard_scan_lsb(generator->cur_moves);
  generator->cur_moves = bitboard_clear_square(generator->cur_moves, dst);

  int player = board_player_to_move(generator->board);
  int opponent = !player;
  // check if move is a capture
  int is_capture = 0;
  int capture_piece = 0;
  board_pos capture_pos = 0;
  if(bitboard_check_square(generator->board->players[opponent], dst)) {
    is_capture = 1;
    capture_piece = board_piece_on_square(generator->board, dst);
    assert(capture_piece != -1);
    capture_pos = dst;
  }
  // check en passant
  if(generator->cur_piece_type == PAWN) {
    board_pos ep_target = board_get_en_passant_target(generator->board);
    if(ep_target != BOARD_POS_INVALID && dst == ep_target) {
      is_capture = 1;
      capture_piece = PAWN;
      capture_pos = en_passant_target_to_pawn_pos(ep_target);
      assert(board_piece_on_square(generator->board, capture_pos) == PAWN);
    }
  }
  // TODO: promotion
  move res = construct_move(generator->board->flags, generator->cur_square, dst, 0, 0, is_capture, capture_piece, capture_pos);
  return res;
}

bitboard board_player_in_check(const board *board, int player) {
  assert(player == WHITE || player == BLACK);
  bitboard king_mask = board->pieces[KING] & board->players[player];
  assert(bitboard_popcount(king_mask) == 1);
  board_pos king_pos = bitboard_scan_lsb(king_mask);
  return board_is_square_attacked(board, king_pos, !player);
}

static move move_gen_next(move_gen *generator, int undo_moves) {
  board_invariants(generator->board);
  int player = board_player_to_move(generator->board);
  int opponent = !player;
  bitboard player_mask = generator->board->players[player];

  if(generator->cur_mode == MOVE_GEN_MODE_NORMAL) {
    // if moves are remaining in bitboard, return them
    if(generator->cur_moves) {
      move next_move = move_gen_next_from_cur_moves(generator);
      // make move
      move_gen_make_move(generator->board, next_move);
      if(board_player_in_check(generator->board, player)) {
        move_gen_unmake_move(generator->board, next_move);
        return move_gen_next(generator, undo_moves);
      }

      if(undo_moves) {
        move_gen_unmake_move(generator->board, next_move);
      }
      return next_move;
    }
    // otherwise, increment to the next square
    do {
      generator->cur_square++;
      if(generator->cur_square >= 64) {
        generator->cur_square = 0;
        generator->cur_piece_type++;
      }
      // if we reached the end of all the pieces, move to castles
      // TODO: actual gen castle moves
      if(generator->cur_piece_type > QUEEN) {
        return MOVE_END;
        break;
      }
    } while(!bitboard_check_square(generator->board->pieces[generator->cur_piece_type] & player_mask, generator->cur_square));
  }
  // generate move mask for the current square and piece type
  assert(generator->cur_mode == MOVE_GEN_MODE_NORMAL);
  generator->cur_moves = move_gen_reg_moves_mask(generator->occupancy_for_sliders, generator->occupancy_for_pawns, generator->cur_piece_type, player, generator->cur_square) & generator->final_moves_mask;
  return move_gen_next(generator, undo_moves);
}

move move_gen_next_move(move_gen *generator) {
  return move_gen_next(generator, 1);
}

move move_gen_make_next_move(move_gen *generator) {
  return move_gen_next(generator, 0);
}

// move generation TODO:
// *[ ] whenever a rook or king moves from their original position, or a rook is captured on its original position, clear the appropriate castling rights
// *[ ] castling legality rules -- see https://en.wikipedia.org/wiki/Castling

void move_gen_pregenerate() {
  move_gen_init_knights();
  move_gen_init_kings();
  move_gen_init_sliders();
  move_gen_init_pawns();
}
