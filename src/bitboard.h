#ifndef BITBOARD_H_INCL
#define BITBOARD_H_INCL
#include "stdint.h"

// bitboard definition + utilities

/**
 * A bitboard is a data structure mapping one bit to one square on the board.
 * For chess, a bitboard stores a single boolean for each square.
 *
 * Implemented as a single 64-bit word. Squares map to bits in little endian order:
 *    a  b  c  d  e  f  g  h
 *   ------------------------
 * 8| 56 57 58 59 60 61 62 63
 * 7| 48 49 50 51 52 53 54 55
 * 6| 40 41 42 43 44 45 46 47
 * 5| 32 33 34 35 36 37 38 39
 * 4| 24 25 26 27 28 29 30 31
 * 3| 16 17 18 19 20 21 22 23
 * 2| 08 09 10 11 12 13 14 15
 * 1| 00 01 02 03 04 05 06 07
 *
 * Bitboards are efficient because we can apply bitwise operations on them, which the cpu can perform quickly.
 */
typedef uint64_t bitboard;

/**
 * A position on a bitboard -- a single square.
 *
 * Implemented as a bitboard index */
typedef uint8_t board_pos;

/**
 * Check if the bit is set for the given square */
int bitboard_check_square(bitboard board, board_pos square);

/**
 * Return the bitboard with the bit for square set */
bitboard bitboard_set_square(bitboard board, board_pos square);

/**
 * Return the bitboard with the bit for square cleared */
bitboard bitboard_clear_square(bitboard board, board_pos square);

/**
 * Return the bitboard with the bit for square flipped */
bitboard bitboard_flip_square(bitboard board, board_pos square);

/**
 * Population count: Count the numbers of bits set (== 1) in the bitboard */
int bitboard_popcount(bitboard board);

/**
 * Get the index of the first bit set in the bitboard (starting from the least significant bit) */
int bitboard_scan_lsb(bitboard board);

bitboard bitboard_shift_n(bitboard board);
bitboard bitboard_shift_s(bitboard board);
bitboard bitboard_shift_w(bitboard board);
bitboard bitboard_shift_e(bitboard board);
bitboard bitboard_shift_nw(bitboard board);
bitboard bitboard_shift_ne(bitboard board);
bitboard bitboard_shift_sw(bitboard board);
bitboard bitboard_shift_se(bitboard board);

/**
 * Print a bitboard on stdout using only ascii */
void bitboard_print(bitboard board);

/**
 * Print a bitboard on stdout using ansi escapes + unicode */
void bitboard_print_pretty(bitboard board);

#define BOARD_POS_INVALID 255

/**
 * Convert x and y coordinates to a board_pos
 * if x and y describe an invalid board position, BOARD_POS_INVALID is returned */
board_pos board_pos_from_xy(int x, int y);

/**
 * Convert a board_pos to x and y coordinates
 * Sets *x and *y to the resulting position */
void board_pos_to_xy(board_pos pos, int *x, int *y);

/**
 * Convert a board_pos to algebraic notation (eg a1, h6, etc)
 * The result will be stored in str, which must be at least 3 bytes */
void board_pos_to_str(board_pos pos, char * str);

/**
 * Convert a pos in algebraic notation to a board_pos */
board_pos board_pos_from_str(const char * str);

/**
 * A board represents the state of a game, including:
 * - piece placement
 * - castling rights
 * - en passant target square
 * - player to move
 *
 * Implementation:
 * A bitboard for each player indicating what squares they have pieces on
 * A bitboard for pawns/rooks/knights/bishops/queens/kings
 * A value with flags for next player to move and en passant target square */
#define WHITE 0
#define BLACK 1

#define KING    0
#define PAWN    1
#define KNIGHT  2
#define ROOK    3
#define BISHOP  4
#define QUEEN   5

#define BOARD_FLAGS_EP_SQUARE   63
#define BOARD_FLAGS_EP_PRESENT  64
#define BOARD_FLAGS_TURN        128

#define BOARD_FLAGS_W_CASTLE_KING   256
#define BOARD_FLAGS_W_CASTLE_QUEEN  512
#define BOARD_FLAGS_B_CASTLE_KING   1024
#define BOARD_FLAGS_B_CASTLE_QUEEN  2048

typedef struct board {
  bitboard players[2];
  bitboard pieces[6];
  uint16_t flags;
} board;

/**
 * check that board is in a consistent state */
void board_invariants(const board *board);

/**
 * initialize a board from a board in FEN notation */
void board_from_fen_str(board *board, const char *fen_string);

/**
 * get the piece at the given square, or -1 if no piece is on the square */
int board_piece_on_square(const board *board, board_pos square);

/**
 * get the player controlling the piece at the given square, or -1 if no piece is on the square */
int board_player_on_square(const board *board, board_pos square);

/**
 * get which player's turn it is for the board */
int board_player_to_move(const board *board);

/**
 * get the en passant target square for the board, or BOARD_POS_INVALID if there is no target square
 *
 * the en passant target square is the square which a pawn skipped while moving forwards 2 squares last turn
 * the en passant target square is directly behind the pawn
 * this turn, that pawn can be capture by moving onto the en passant target square
 *
 * if no pawn was moves forwards 2 squares last turn, then there is no en passant target square
 */
board_pos board_get_en_passant_target(const board *board);

/**
 * check if player can castle (side should be either KING or QUEEN) */
int board_can_castle(const board *board, int player, int side);

/**
 * print a board on stdout using ony ascii characters */
void board_print(const board *board);

/**
 * print a board on stdout using ansi escapes + unicode */
void board_print_pretty(const board *board);

/**
 * Convert a piece character to a piece index
 * pawn: P + p, knight: N + n, bishop: B + b, rook: R + r, queen: Q + q, king: K + k
 */
int board_piece_char_to_piece(char c);
/**
 * Convert a piece character to a player index
 * white: P, N, B, R, Q, K
 * black: p, n, b, r, q, k
 */
int board_piece_char_to_player(char c);

/**
 * convert a piece and player index into a character representing that piece
 * white: P, N, B, R, Q, K
 * black: p, n, b, r, q, k
 */
char board_piece_char_from_piece_player(int piece, int player);

#endif
