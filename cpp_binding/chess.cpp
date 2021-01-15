#include "chess.hpp"
#include <iostream>
#include <sstream>

#include <boost/beast/core.hpp>
#include <boost/beast/websocket.hpp>
#include <boost/asio/connect.hpp>
#include <boost/asio/ip/tcp.hpp>
#include <boost/format.hpp>

extern "C" {
#include "chess-util.h"
}

namespace chess {

int PieceTypeToInternalValue(PieceType piece) {
  return static_cast<int>(piece) & 7;
}

PieceType PieceTypeFromInternalValue(int value) {
  return PieceType(value & 7);
}

BoardPos::BoardPos(uint8_t pos): pos(pos) {}

BoardPos::BoardPos(int x, int y): pos(board_pos_from_xy(x, y)) {}

int BoardPos::x() const { return board_pos_to_x(pos); }
int BoardPos::y() const { return board_pos_to_y(pos); }

std::string BoardPos::toString() const {
  char str[3];
  board_pos_to_str(pos, str);
  return std::string(str);
}

BoardPos BoardPos::fromString(const std::string &str) {
  return BoardPos(board_pos_from_str(str.c_str()));
}

bool BoardPos::isInvalid() const {
  return pos >= 64;
}

BoardPos BoardPos::invalid() { return BOARD_POS_INVALID; }

uint8_t BoardPos::getInternalValue() const { return pos; }

BoardPos BoardPos::fromInternalValue(uint8_t pos) {
  return BoardPos(pos);
}

Bitboard::Bitboard(uint64_t value): value(value) {}

bool Bitboard::checkSquare(BoardPos pos) const { return bitboard_check_square(value, pos.getInternalValue()); }

int Bitboard::count() const { return bitboard_popcount(value); }

bool Bitboard::hasAnySet() const { return value != 0; }

BoardPos Bitboard::firstPosSet() const {
  return BoardPos::fromInternalValue(bitboard_scan_lsb(value));
}
Bitboard Bitboard::withPosSet(BoardPos posToSet) const { return Bitboard(bitboard_set_square(value, posToSet.getInternalValue())); }

Bitboard Bitboard::withPosClear(BoardPos posToClear) const { return Bitboard(bitboard_clear_square(value, posToClear.getInternalValue())); }

Bitboard Bitboard::withPosFlipped(BoardPos posToFlip) const {
  return Bitboard(bitboard_flip_square(value, posToFlip.getInternalValue()));
}

uint64_t Bitboard::getInternalValue() const { return value; }

void Bitboard::print() const {
  bitboard_print(value);
}

void Bitboard::prettyPrint() const {
  bitboard_print_pretty(value);
}

Bitboard Bitboard::fromInternalValue(uint64_t value) { return Bitboard(value); }

bool Move::isEnd() { return move == MOVE_END; }

Move Move::end() { return Move::fromInternalValue(MOVE_END); }

BoardPos Move::sourceSquare() const { return BoardPos::fromInternalValue(move_source_square(move)); }

BoardPos Move::destinationSquare() const { return BoardPos::fromInternalValue(move_destination_square(move)); }

bool Move::isPromotion() const { return move_is_promotion(move); }

PieceType Move::promotionPieceType() const { return PieceType(move_promotion_piece(move)); }

bool Move::isCapture() const { return move_is_capture(move); }

PieceType Move::capturePieceType() const { return PieceType(move_capture_piece(move)); }

BoardPos Move::captureSquare() const { return BoardPos::fromInternalValue(move_capture_square(move)); }

bool Move::isCastle() const { return move_is_castle(move); }

std::string Move::toString() const {
  char str[6];
  move_to_str(move, str);
  return std::string(str);
}

Move Move::fromString(const std::string &moveString, const Board &board) {
  return Move::fromInternalValue(move_from_str(moveString.c_str(), board.getInternalValue()));
}

Move Move::fromInternalValue(uint64_t internal) { return Move(internal); }

Move::Move(uint64_t move): move(move) {}

uint64_t Move::getInternalValue() const { return move; }

Move::Move(BoardPos src, BoardPos dst, bool is_promote, PieceType promote_piece,
           const Board &board) {
  move = move_new(src.getInternalValue(), dst.getInternalValue(), is_promote, static_cast<int>(promote_piece), board.getInternalValue());
}

Move MoveGenerator::nextMove() {
  return Move::fromInternalValue(move_gen_next_move(&gen));
}

Move MoveGenerator::makeNextMove() { return Move::fromInternalValue(move_gen_make_next_move(&gen)); }

bool MoveGenerator::isCheckmate() { return move_gen_is_checkmate(&gen); }

bool MoveGenerator::isStalemate() { return move_gen_is_stalemate(&gen); }

MoveGenerator::MoveGenerator(Board &board): gen() {
  move_gen_init(&gen, board.getInternalValue());
}

Board::Board(const std::string &fenString): board() {
  board_from_fen_str(&board, fenString.c_str());
}

std::string Board::toString() const {
  char str[87];
  board_to_fen_str(&board, str);
  return std::string(str);
}

PieceType Board::pieceTypeOnSquare(BoardPos square) const {
  return PieceType(board_piece_on_square(&board, square.getInternalValue()));
}

Player Board::playerOnSquare(BoardPos square) const { return Player(board_player_on_square(&board, square.getInternalValue())); }

Player Board::playerToMove() const {
  return Player(board_player_to_move(&board));
}

bool Board::playerInCheck(Player player) const { return board_player_in_check(&board, static_cast<int>(player)); }

Bitboard Board::squareThreatened(BoardPos square, Player attackingPlayer) {
  return Bitboard::fromInternalValue(board_is_square_attacked(&board, square.getInternalValue(),static_cast<int>(attackingPlayer)));
}

void Board::makeMove(Move move) {
  board_make_move(&board, move.getInternalValue());
}
void Board::unmakeMove(Move move) {
  board_unmake_move(&board, move.getInternalValue());
}

MoveGenerator Board::getMoveGenerator() {
  return MoveGenerator(*this);
}

bool Board::playerToMoveIsCheckmate() {
  return board_is_checkmate(&board);
}

bool Board::playerToMoveIsStalemate() {
  return board_is_stalemate(&board);
}

void Board::print() const {
  board_print(&board);
}

void Board::prettyPrint() const {
  board_print_pretty(&board);
}

BoardPos Board::enPassantTarget() const {
  return BoardPos::fromInternalValue(board_get_en_passant_target(&board));
}

bool Board::hasCastlingRights(Player player, PieceType side) const {
  return board_can_castle(&board, static_cast<int>(player),
                          static_cast<int>(side));
}

struct __chess_util_board *Board::getInternalValue(){
  return &board;
}

const struct __chess_util_board *Board::getInternalValue() const {
  return &board;
}

Bitboard Board::playerBitboard(Player player) const {
  return Bitboard::fromInternalValue(board.players[static_cast<int>(player)]);
}
Bitboard Board::pieceBitboard(Player player, PieceType piece) const {
  return Bitboard::fromInternalValue(board.players[static_cast<int>(player)] & board.pieces[static_cast<int>(piece)]);
}

int Board::fullTurnNumber() const { return board_get_full_turn_number(&board); }

void connectToServer(std::string url, const std::string& port, const std::string& apikey, const std::string& name, const std::function<std::pair<Move, std::unordered_map<std::string, std::string>>(Board &)> &function) {
  move_gen_pregenerate();

  try {
    boost::asio::io_context ioc;
    boost::asio::ip::tcp::resolver resolver{ioc};
    boost::beast::websocket::stream<boost::asio::ip::tcp::socket> ws{ioc};

    // dns lookup on host
    auto const resolve_res = resolver.resolve(url, port);
    // connect
    auto ep = boost::asio::connect(ws.next_layer(), resolve_res);
    url += ":" + std::to_string(ep.port());
    ws.handshake(url, "/");

    // send apikey and name commands
    ws.write(boost::asio::buffer(str(boost::format("apikey %1%") % apikey)));
    ws.write(boost::asio::buffer(str(boost::format("name %1%") % name)));

    while (true) {
      boost::beast::flat_buffer msg;
      ws.read(msg);
      std::string msg_string = boost::beast::buffers_to_string(msg.data());

      if(msg_string.starts_with("position")) {
        std::string fen_board = msg_string.substr(msg_string.find(' ') + 1);
        auto board = Board(fen_board);
        // get move from function
        auto function_res = function(board);
        auto move_to_make = function_res.first;
        auto debug_info = function_res.second;
        // make move
        ws.write(boost::asio::buffer(str(boost::format("move %1%") % move_to_make.toString())));
        // send debug info
        std::ostringstream debug_out;
        debug_out << "info ";
        for(auto & dentry: debug_info) {
          debug_out << dentry.first << " " << dentry.second << "`";
        }
        ws.write(boost::asio::buffer(debug_out.str()));
      } else if (msg_string.starts_with("error")) {
        std::cerr << "Error from server: " << msg_string << "\n";
      }
    }
  } catch(std::exception const& e) {
    std::cerr << "Error: " << e.what() << std::endl;
  }
}

}