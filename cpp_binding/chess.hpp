#ifndef HPP_CHESS_INCL
#define HPP_CHESS_INCL

#include <cstdint>
#include <string>
#include <functional>
#include <unordered_map>

extern "C" {
#ifndef H_CHESS_UTIL_TYPEDEFS_INCL
#define H_CHESS_UTIL_TYPEDEFS_INCL
typedef uint64_t __chess_util_bitboard;
typedef uint8_t __chess_util_board_pos;

struct __chess_util_board {
  __chess_util_bitboard players[2];
  __chess_util_bitboard pieces[6];
  uint16_t flags;
};

typedef uint64_t __chess_util_move;

struct __chess_util_move_gen {
  struct __chess_util_board *board;
  __chess_util_bitboard occupancy_for_sliders;
  __chess_util_bitboard occupancy_for_pawns;
  __chess_util_bitboard final_moves_mask;
  uint8_t cur_mode;
  uint8_t cur_piece_type;
  uint8_t cur_square;
  uint8_t cur_promotion;
  __chess_util_bitboard cur_moves;
  uint8_t done;
  uint8_t hit_move;
};
#endif
}

/*! \file */

namespace chess {

/**
 * A position on a chessboard.
 * A position's x coordinate corresponds to it's rank (a-f) and it y coordinate corresponds to it's file (1-8)
 */
class BoardPos {
private:
  uint8_t pos;

  BoardPos(uint8_t pos);

public:
  /**
   * Construct a board position from the given x and y coordinates
   */
  BoardPos(int x, int y);

  /**
   * Get the x coordinate of the position. It is in the range 0 - 7 inclusive. The x coordinate corresponds to the position's rank on the chessboard.
   */
  int x() const;

  /**
   * Get the y coordinate of the position. It is in the range 0 - 7 inclusive. The y coordinate corresponds to the position's file on the chessboard.
   */
  int y() const;

  /**
   * Get a string representing the position in rank and file notation.
   * For example, the following positions correspond to the following strings:
   * (0, 0) -> a1
   * (1, 0) -> b1
   * (7, 0) -> h1
   * (0, 1) -> a2
   * (0, 7) -> a8
   * (7, 7) -> h8
   */
  std::string toString() const;

  /**
   * Convert a string in rank and file notation to a board position.
   */
  static BoardPos fromString(const std::string &str);

  /**
   * Check if the board position is invalid (ie off the board)
   */
  bool isInvalid() const;

  /**
   * Construct an invalid board position
   */
  static BoardPos invalid();

  /**
   * Get the underlying integer representation of the position.
   * The underlying representation is:
   *    (x & 7) + (y & 7) << 3
   *
   * You probably don't need this.
   */
  uint8_t getInternalValue() const;

  /**
   * Construct a BoardPos from an underlying value. You probably don't need this.
   */
  static BoardPos fromInternalValue(uint8_t pos);
};

/**
 * A bitboard is a data structure that maps each position on a chessboard to a single bit of information. IE -- It contains either a 1 or 0 for each square on the board.
 *
 * A bitboard does not represent the state of a game -- it represents one aspect about that game for each square.
 */
class Bitboard {
private:
  uint64_t value;

public:

  /**
   * Construct a bitboard from a 64 bitboard value */
  Bitboard(uint64_t value);

  /**
   * Check if the bit for a square is set (ie is a 1). Returns true if the bit for the square is set to 1, false otherwise.
   */
  bool checkSquare(BoardPos pos) const;

  /**
   * Return the total number of bits that are set to 1 in the bitboard.
   */
  int count() const;

  /**
   * Return true if any bits are set to 1 in the bitboard, or false if all bits are 0 */
  bool hasAnySet() const;

  /**
   * Return the location of the first bit set to 1 in the bitboard, starting at a1 -> b1 -> ... -> h1 -> a2 -> b2 ... -> h8. If no bits are set to 1, the return value is undefined.
   */
  BoardPos firstPosSet() const;

  /**
   * Construct a new bitboard that is a copy of this bitboard with the given position set to 1.
   */
  Bitboard withPosSet(BoardPos posToSet) const;

  /**
   * Construct a new bitboard that is a copy of this bitboard with the given position set to 0.
   */
  Bitboard withPosClear(BoardPos posToClear) const;

  /**
   * Construct a new bitboard that is a copy of this bitboard with the given position flipped (ie flipped from a 0 to 1 or 1 to 0).
   */
  Bitboard withPosFlipped(BoardPos posToFlip) const;

  /**
   * Get the underlying integer representation of the bitboard.
   * The underlying representation is a 64 bit integer, where each bit corresponds to a position.
   *
   * You probably don't need this. If you want to do bitwise manipulation of bitboards, the bitwise operations are overridden on the Bitboard class itself.
   */
  uint64_t getInternalValue() const;

  static Bitboard fromInternalValue(uint64_t value);

  /**
   * Print the bitboard onto stdout
   */
  void print() const;

  /**
   * Print the bitboard onto stdout, using ansi escape sequences
   */
  void prettyPrint() const;

  friend Bitboard operator&(Bitboard lhs, Bitboard rhs) {
    return Bitboard(lhs.value & rhs.value);
  }

  friend Bitboard operator|(Bitboard lhs, Bitboard rhs) {
    return Bitboard(lhs.value | rhs.value);
  }

  friend Bitboard operator^(Bitboard lhs, Bitboard rhs) {
    return Bitboard(lhs.value ^ rhs.value);
  }

  friend Bitboard operator>>(Bitboard lhs, int rhs) {
    return Bitboard(lhs.value >> rhs);
  }

  friend Bitboard operator<<(Bitboard lhs, int rhs) {
    return Bitboard(lhs.value << rhs);
  }

  Bitboard& operator&=(Bitboard rhs)
  {
    value &= rhs.value;
    return *this;
  }

  Bitboard& operator|=(Bitboard rhs)
  {
    value |= rhs.value;
    return *this;
  }

  Bitboard& operator^=(Bitboard rhs)
  {
    value ^= rhs.value;
    return *this;
  }

  Bitboard& operator>>=(int rhs)
  {
    value >>= rhs;
    return *this;
  }

  Bitboard& operator<<=(int rhs)
  {
    value <<= rhs;
    return *this;
  }
};

inline bool operator==(const Bitboard& lhs, const Bitboard& rhs){
  return lhs.getInternalValue() == rhs.getInternalValue();
}

inline bool operator!=(const Bitboard& lhs, const Bitboard& rhs){
  return !(lhs == rhs);
}

/**
 * A player in a chess game
 */
enum class Player {
  None = -1,
  White = 0,
  Black = 1,
};

/**
 * A type of piece in a chess game
 */
enum class PieceType {
  None = -1,
  King = 0,
  Pawn = 1,
  Knight = 2,
  Rook = 3,
  Bishop = 4,
  Queen = 5,
};

/**
 * Convert a PieceType to a int representation.
 * The resulting int has three bits.
 */
int PieceTypeToInternalValue(PieceType piece);

/**
 * Construct a PieceType from its int representation.
 */
PieceType PieceTypeFromInternalValue(int value);

// forward declare
class Board;

/**
 * A move that can be made of a chessboard. Move represents a half turn -- one player's move.
 */
class Move {
private:
  uint64_t move;

  Move(uint64_t move);

public:
  /**
   * Construct a new move from a source position, destination position, and promotion information.
   * @param[src] src is the source square of the move (where the piece starts).
   * @param[dst] dst is the destination square of the move (where the piece ends).
   * @param[is_promote] is_promote should be true if the move is a pawn promotion.
   * @param[promote_piece] promote_piece is the type of piece the pawn is being promoted to. If is_promote is false, promote_piece is ignored.
   * @param[board] board is the board that the move will be later made on. This does not make the move -- it simply needs to know what board the move will be made on so it can encode extra information (capturing, castling, etc) in the returned Move.
   */
  Move(BoardPos src, BoardPos dst, bool is_promote, PieceType promote_piece, const Board& board);

  /**
   * Check if this move is the end of the move generator's legal moves.
   * If isEnd() is true, the move isn't legal -- it just represents that the move generator that made it is out of moves.
   */
  bool isEnd();

  /**
   * Construct a move with the isEnd() condition met
   */
  static Move end();

  /**
   * get the source square for the move (ie where the piece being moved starts)
   */
   BoardPos sourceSquare() const;

   /**
    * get the destination square for the move (ie where the piece being moved ends up)
    */
   BoardPos destinationSquare() const;

   /**
    * return true if the move is a pawn promotion
    */
   bool isPromotion() const;

   /**
    * If the move is a pawn promotion, return what type of piece it is being promoted to. Return PieceType::None otherwise
    */
   PieceType promotionPieceType() const;

   /**
    * Return move if the move is a capture.
    */
   bool isCapture() const;

   /**
    * If the move is a capture, return what type of piece is being capture. Return PieceType::None otherwise.
    */
   PieceType capturePieceType() const;

   /**
    * If the move is a capture, return what square is being captured. Return BoardPos::invalid() otherwise.
    *
    * The capture square is the same as the destination square, except for en passant captures.
    */
   BoardPos captureSquare() const;

   /**
    * Return true if the move is castling, false otherwise.
    *
    * Castling source and destinations are the source and destination of the king. For example, in white kingside castling, the source is e1 and the destination g1.
    */
   bool isCastle() const;

   /**
    * Convert the move to a string in pure algebraic notation.
    *
    * Pure algebraic notation is of the form:
    * <src><dst><promote?>
    *
    * For example:
    * e2e4 -- moving a piece from square e2 to e4
    * a7a8q -- moving a pawn from a7 to a8 and promoting to a queen
    *
    * Captures don't have a special notation.
    * En passant doesn't have a special notation.
    */
   std::string toString() const;

   /**
    * Convert a move from a pure algebraic notation string to Move.
    * board is the board on which the move will be made.
    */
   static Move fromString(const std::string& moveString, const Board &board);

   static Move fromInternalValue(uint64_t internal);

   uint64_t getInternalValue() const;
};

/**
 * A class that iterates through all of the legal moves that can be made on a board.
 *
 */
class MoveGenerator {
private:
  __chess_util_move_gen gen;

public:
  /**
   * Construct a move generator to iterate through the legal moves for board
   */
  MoveGenerator(Board &board);

  /**
   * Get the next legal moves that can be made. If the move generator is out of legal moves, return Move::end().
   */
  Move nextMove();

  /**
   * Get the next legal moves that can be made, and make it on the board. If the move generator is out of legal moves, return Move::end().
   *
   * You MUST call Board::unmakeMove before advancing the move generator further.
   * Calling makeNextMove() is slightly faster than calling nextMove() then Board::makeMove() if you want to evaluate the effect of the move.
   */
  Move makeNextMove();

  /**
   * Check if the player to move on the board is in checkmate.
   *
   * You can only call this once the move generator has been exhausted (once nextMove or makeNextMove has returned Move::end()).
   */
  bool isCheckmate();

  /**
   * Check if the player to move on the board is in stalemate.
   *
   * You can only call this once the move generator has been exhausted (once nextMove or makeNextMove has returned Move::end()).
   */
  bool isStalemate();
};

/**
 * A chess board. Board includes the layout of the pieces on the board and the current player to move
 */
class Board {
private:
  __chess_util_board board;

public:
  /**
   * Construct a board from a string in Forsyth-Edwards Notation (FEN).
   */
  Board(const std::string& fenString);

  /**
   * Convert the board to a string in Forsyth-Edwards Notation (FEN). The halfmove and turn counters will contain fake values (Board doesn't track them).
   */
  std::string toString() const;

  /**
   * Get the type of piece on the given square, or PieceType::None if no piece is on the square.
   */
  PieceType pieceTypeOnSquare(BoardPos square) const;

  /**
   * Get the player that owns the piece on the given square, or Player::None if no piece is on the square.
   */
  Player playerOnSquare(BoardPos square) const;

  /**
   * Get a bitboard with bits set for each square that has a piece on it owned by the given player (of any type).
   */
  Bitboard playerBitboard(Player player) const;

  /**
   * Get a bitboard with bits set for each square that has a player has a piece of the given type on.
   */
  Bitboard pieceBitboard(Player player, PieceType piece) const;

  /**
   * Get which player's turn it is to move
   */
  Player playerToMove() const;

  /**
   * Get the turn number for the board. The turn number starts at 1 and is incremented when a full turn is made (both players make a move)
   */
  int fullTurnNumber() const;

  /**
   * Check if the given player is currently in check.
   */
  bool playerInCheck(Player player) const;

  /**
   * Check if a given square is threatened by attackingPlayer. IE -- if attackingPlayer could capture square on their next turn.
   *
   * Returns a bitboard with bits set for each piece that is threatening the square.
   */
  Bitboard squareThreatened(BoardPos square, Player attackingPlayer);

  /**
   * Make the given move. This modifies the board.
   */
  void makeMove(Move move);

  /**
   * Unmake the given move. The move to unmake has to have been the most recently made move on the board. This modifies the board.
   */
  void unmakeMove(Move move);

  /**
   * Construct a move generator for the board. The move generator will iterate through all the legal moves that can be made on the board.
   */
  MoveGenerator getMoveGenerator();

  /**
   * Check if the player to move is in checkmate.
   *
   * This method is slow. If you are iterating through a MoveGenerator, consider using MoveGenerator::isCheckmate instead.
   */
  bool playerToMoveIsCheckmate();

  /**
   * Check if the player to move is in stalemate.
   *
   * This method is slow. If you are iterating through a MoveGenerator, consider using MoveGenerator::isStalemate instead.
   */
  bool playerToMoveIsStalemate();

  /**
   * Print the board on stdout
   */
  void print() const;

  /**
   * Print the board on stdout using ansi escape codes and unicode chess characters
   */
  void prettyPrint() const;

  /**
   * Get the en passant target square, or BoardPos::invalid() if there is no en passant target square.
   *
   * The en passant target square is the square which a pawn skipped last turn while moving forwards two squares, and is therefore vulnerable to en passant capture. If a pawn didn't move forward two squares last turn, these is no en passant target.
   */
   BoardPos enPassantTarget() const;

   /**
    * Check if the player has castling rights on side. side should be either PieceType::King or PieceType::Queen. Having castling rights does not mean the player can castle -- rather, it means only they have not moved their king and rook */
   bool hasCastlingRights(Player player, PieceType side) const;

   struct __chess_util_board * getInternalValue();

  const struct __chess_util_board * getInternalValue() const;
};

/**
 * Connect to a codekata chess server at the given url and port, send apikey and name, and block until the server requests a move.
 *
 * When a move is requested, function will be called. It will be passed the current board, and should return the move to make and a map of debug information to pass to the server (debug info can be empty)
 */
void connectToServer(std::string url, const std::string& port, const std::string& apikey, const std::string& name, const std::function<std::pair<Move, std::unordered_map<std::string, std::string>>(Board &)> &function);

}
#endif