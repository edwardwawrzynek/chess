#ifndef MOVE_GEN_H_INCL
#define MOVE_GEN_H_INCL

#include "bitboard.h"

/**
 * pre generate move generation lookup tables */
void move_gen_pregenerate();

/**
 * move represents a single piece move on a board
 *
 * Implementation:
 * move has to be reversible
 * bits 0-15  : board's previous flags
 * bits 16-21 : move source
 * bits 22-27 : move destination
 * bit  28    : set if the move is a promotion
 * bits 31-29 : promotion piece value
 * bit  32    : set if the move is a capture
 * bits 33-35 : type of captured piece
 * bits 36-41 : square capture piece was on (may be different from move destination due to en passant)
 */
typedef uint64_t move;

#define MOVE_END                  0xffffffffffffffffULL

/**
 * get the source square of a move (where a piece is being moved from ) */
board_pos move_source_square(move move);
/**
 * get the destination square of a move (where a piece is being moved to) */
board_pos move_destination_square(move move);

/**
 * check if a move is a promotion */
int move_is_promotion(move move);
/**
 * if a move is a promotion, return what piece it is being promoted to
 * return -1 otherwise */
int move_promotion_piece(move move);

/**
 * check if a move is a capture */
int move_is_capture(move move);
/**
 * if a move is a capture, return what piece type is being captured
 * return -1 otherwise */
int move_capture_piece(move move);
/**
 * if a move is a capture, return what square is being captured
 * return BOARD_POS_INVALID otherwise
 * in most cases, this is the same as the move's destination
 * for en passant, it is different */
board_pos move_capture_square(move move);

/**
 * convert a move to a string in pure algebraic notation, which is of the form:
 * <src><dst><promote?>
 * for example, moving a piece (of any type) from e2 to e4 is:
 * e2e4
 * moving a pawn from a7 to a8 and promoting to a queen is:
 * a7a8q
 * a7a8n (promoting to knight)
 * a7a8b (promoting to bishop)
 * a7a8r (promoting to rook)
 *
 * res_str must have 6 bytes allocated */
void move_to_str(move move, char *res_str);

/**
 * construct a move from a string and a board
 * board must be a legal board for the move to be applied on (but the move will not actually be made on it) */
move move_from_str(char *move_str, const board *board);

/**
 * move_gen contains the state of the move generation algorithm */
typedef struct move_gen {
  // board having move generated for
  board *board;
  // occupancy bitboards for sliders + pawns
  // the occupancy for pawns includes the en passant target, occupancy for sliders doesn't
  bitboard occupancy_for_sliders;
  bitboard occupancy_for_pawns;
  // final mask to & with moves
  // for all moves, this should be ~us
  // for attacks, this should be them
  bitboard final_moves_mask;
  // current move being generated
  uint8_t cur_mode;
  uint8_t cur_piece_type;
  uint8_t cur_square;
  uint8_t cur_promotion;
  bitboard cur_moves;
} move_gen;


/**
 * initialize a move_gen structure for a given board
 * if captures_only, the generator will only generate moves that capture pieces */
void move_gen_init(move_gen *move_gen, board *board, int captures_only);

/**
 * check if a square on the board is attacked by a certain player
 * return a bitboard with each square set that is threatening the square
 * if the square is not attacked, the bitboard is 0 */
bitboard board_is_square_attacked(const board *board, board_pos square, int attacking_player);

/**
 * check if player is in check (ie -- player's king is attacked by opponent) */
bitboard board_player_in_check(const board *board, int player);

/**
 * get the next move from the move generator
 * if no more moves are available, the method returns MOVE_END */
move move_gen_next_move(move_gen *generator);

/**
 * get the next move from the move generator and apply it to board
 * if no more moves are available, the method returns MOVE_END
 * you MUST undo the move before calling move_gen_make_next_move or move_gen_next_move again */
move move_gen_make_next_move(move_gen *generator);

/**
 * make the given move on the given board
 * mutates board */
void move_gen_make_move(board *board, move move);

/**
 * undo the given move on the given board
 * the move must have been previously made on the board
 * mutates board */
void move_gen_unmake_move(board *board, move move);

#endif
