#include "chess.hpp"
#include <iostream>
#include <utility>
#include <unordered_map>

using namespace chess;
extern "C" {
#include "chess-util.h"
}

/**
 * Sample C++ Codekata Chess Client
 *
 * This sample implements a very basic minimax
 */

// a value which eval is guaranteed to return less than
#define EVAL_INFINITY 100000000

// evaluate a board from white's perspective
int eval(const Board &board) {
  // evaluate based on material
  return (
      // white
      1 * board.pieceBitboard(Player::White, PieceType::Pawn).count() +
      3 * board.pieceBitboard(Player::White, PieceType::Knight).count() +
      3 * board.pieceBitboard(Player::White, PieceType::Bishop).count() +
      5 * board.pieceBitboard(Player::White, PieceType::Rook).count() +
      9 * board.pieceBitboard(Player::White, PieceType::Queen).count() +
      // black
      -1 * board.pieceBitboard(Player::Black, PieceType::Pawn).count() +
      -3 * board.pieceBitboard(Player::Black, PieceType::Knight).count() +
      -3 * board.pieceBitboard(Player::Black, PieceType::Bishop).count() +
      -5 * board.pieceBitboard(Player::Black, PieceType::Rook).count() +
      -9 * board.pieceBitboard(Player::Black, PieceType::Queen).count() +

      // reward control of the center 4 squares
      // 0x1818000000 was generated by the Bitboard Construction Tool
      2 * (board.playerBitboard(Player::White) & Bitboard(0x1818000000)).count() +
      -2 * (board.playerBitboard(Player::Black) & Bitboard(0x1818000000)).count()
  );
}

// run a minimax on the board and return a pair of (score, best move)
// color is 1 if white, -1 if black
std::pair<int, Move> minimax(Board &board, int depth, int color) {
  // if we reached bottom depth, evaluate
  if(depth == 0) {
    int value = color * eval(board);
    return std::pair(value, Move::end());
  }

  // otherwise, generate all moves and score them
  MoveGenerator moveGenerate = board.getMoveGenerator();
  // get the first legal move and apply it
  Move currentMove = moveGenerate.makeNextMove();
  // current best score and move found so far
  int value = -EVAL_INFINITY;
  Move bestMove = currentMove;
  // loop through all moves
  while(!currentMove.isEnd()) {
    // makeNextMove() already made the move for us, so we just visit the board
    auto childVisit = minimax(board, depth - 1, -color);
    // we don't care about the best child move, so just get the score
    // inverted because child score was for opposite color
    auto childScore = -childVisit.first;

    if(childScore > value) {
      value = childScore;
      bestMove = currentMove;
    }

    // unapply the current move
    board.unmakeMove(currentMove);
    // and make the next legal move
    currentMove = moveGenerate.makeNextMove();
  }

  // check for checkmate (score -infinity) or stalemate (score 0)
  if(moveGenerate.isCheckmate()) {
    value = -EVAL_INFINITY;
  } else if(moveGenerate.isStalemate()) {
    value = 0;
  }

  return std::pair(value, bestMove);
}

std::pair<Move, std::unordered_map<std::string, std::string>> findMove(Board &board) {
  std::cout << "---- Board To Make Move On ----" << std::endl;
  std::cout << board.toString() << std::endl;
  board.print();

  int color = board.playerToMove() == Player::White ? 1 : -1;
  auto minimaxRes = minimax(board, 4, color);
  std::cout << "making move: " << minimaxRes.second.toString() << ", with score " << minimaxRes.first << std::endl;

  // construct debug info to send back to server
  // we can send any key value pair
  // the "eval" key is interpreted specially to mean the value we gave the move we are making
  std::unordered_map<std::string, std::string> debug_info {
      {"eval", std::to_string(minimaxRes.first)},
      {"depth", "4"},
      // add more here
  };

  return std::pair(minimaxRes.second, std::move(debug_info));
}

int main(int argc, char *argv[]) {
  if(argc < 5) {
    std::cerr << "usage: " << argv[0] << " host port apikey name" << std::endl;
    std::cerr << "example: " << argv[0] << " codekata-chess.herokuapp.com 80 API_KEY Computer" << std::endl;
    return 1;
  }
  auto host = std::string(argv[1]);
  auto port = std::string(argv[2]);
  auto apikey = std::string(argv[3]);
  auto name = std::string(argv[4]);

  connectToServer(host, port, apikey, name, findMove);
  return 0;
}