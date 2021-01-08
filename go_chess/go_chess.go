package go_chess

// #cgo LDFLAGS: -lchess-util
// #include <stdlib.h>
// #include <string.h>
// #include "chess-util.h"
import "C"

import (
	"fmt"
	"log"
	"net/url"
	"strings"
	"unsafe"

	"github.com/gorilla/websocket"
)

// BoardPos represents a position (ie a square) on a chess board.
// A position's x coordinate cooresponds to it's rank (a-f) and its y coordinate cooresponds to it's file (1-8)
type BoardPos C.board_pos

// InvalidBoardPos is a position not present on the board. It is used to indicate that something is not present on a board
const InvalidBoardPos BoardPos = BoardPos(255)

// NewBoardPos constructs a board position from the given x and y coordinates
func NewBoardPos(x int, y int) BoardPos {
	return BoardPos(C.board_pos_from_xy(C.int(x), C.int(y)))
}

// XY converts a BoardPos to x and y coordinates, each in the range 0-7 inclusive
func (pos BoardPos) XY() (int, int) {
	return int(C.board_pos_to_x(C.board_pos(pos))), int(C.board_pos_to_y(C.board_pos(pos)))
}

// X converts a BoardPos to a x coordinate in the range 0-7 inclusive
func (pos BoardPos) X() int {
	return int(C.board_pos_to_x(C.board_pos(pos)))
}

// Y converts a BoardPos to a y coordinate in the range 0-7 inclusive
func (pos BoardPos) Y() int {
	return int(C.board_pos_to_x(C.board_pos(pos)))
}

// ToString converts a BoardPos to a string in rank and file notation (ex: a1, b5, h8)
func (pos BoardPos) ToString() string {
	cs := C.CString("__")
	defer C.free(unsafe.Pointer(cs))
	C.board_pos_to_str(C.board_pos(pos), cs)
	return C.GoString(cs)
}

// BoardPosFromString constructs a BoardPos from a string in rank and file notation (ex: a1. b5, h8)
func BoardPosFromString(str string) BoardPos {
	cs := C.CString(str)
	defer C.free(unsafe.Pointer(cs))
	return BoardPos(C.board_pos_from_str(cs))
}

// Bitboard is a data structure that maps each position on a chessboard to a single bit of information. IE -- It contains either a 1 or 0 for each square on the board.
// Bitboard does not represent the state of a game -- it represents one aspect about that game for each square.
type Bitboard C.bitboard

// CheckSquare checks if the given square is set (is a 1). Returns true if the square is a 1, false otherwise.
func (bb Bitboard) CheckSquare(pos BoardPos) bool {
	return C.bitboard_check_square(C.bitboard(bb), C.board_pos(pos)) != 0
}

// Count counts the number of bits that are set to 1 in the bitboard
func (bb Bitboard) Count() int {
	return int(C.bitboard_popcount(C.bitboard(bb)))
}

// HasAnySet returns true if the bitboard has any bits that are set to 1, or false if all bits are set to 0
func (bb Bitboard) HasAnySet() bool {
	return C.bitboard(bb) != 0
}

// FirstPosSet returns the location of the first bit set to 1 in the bitboard, starting at a1 -> b1 -> ... -> h1 -> a2 -> b2 -> ... -> h8. If no bits are set to 1, the return value is undefined
func (bb Bitboard) FirstPosSet() BoardPos {
	return BoardPos(C.bitboard_scan_lsb(C.bitboard(bb)))
}

// WithPosSet returns a copy of the bitboard with the given bit set to 1
func (bb Bitboard) WithPosSet(posToSet BoardPos) Bitboard {
	return Bitboard(C.bitboard_set_square(C.bitboard(bb), C.board_pos(posToSet)))
}

// WithPosClear returns a copy of the bitboard with the given bit set to 0
func (bb Bitboard) WithPosClear(posToClear BoardPos) Bitboard {
	return Bitboard(C.bitboard_clear_square(C.bitboard(bb), C.board_pos(posToClear)))
}

// WithPosFlipped returns a copy of the bitboard with the given bit flipped (ie: flipped from 0 to 1 or 1 to 0)
func (bb Bitboard) WithPosFlipped(posToFlip BoardPos) Bitboard {
	return Bitboard(C.bitboard_flip_square(C.bitboard(bb), C.board_pos(posToFlip)))
}

// Print prints the bitboard on stdout
func (bb Bitboard) Print() {
	C.bitboard_print(C.bitboard(bb))
}

// PrettyPrint prints the bitboard on stdout, using ansi escape sequences
func (bb Bitboard) PrettyPrint() {
	C.bitboard_print_pretty(C.bitboard(bb))
}

// And applies a bitwise and (intersection) between lhs and rhs and returns the result
func (lhs Bitboard) And(rhs Bitboard) Bitboard {
	return Bitboard(C.bitboard(lhs) & C.bitboard(rhs))
}

// Or applies a bitwise or (union) between lhs and rhs and returns the result
func (lhs Bitboard) Or(rhs Bitboard) Bitboard {
	return Bitboard(C.bitboard(lhs) | C.bitboard(rhs))
}

// Xor applies a bitwise xor between lhs and rhs and returns the result
func (lhs Bitboard) Xor(rhs Bitboard) Bitboard {
	return Bitboard(C.bitboard(lhs) ^ C.bitboard(rhs))
}

// LeftShift shifts lhs left by rhs places and returns the result
func (lhs Bitboard) LeftShift(rhs int) Bitboard {
	return Bitboard(C.bitboard(lhs) << rhs)
}

// Righthift shifts lhs right by rhs places and returns the result
func (lhs Bitboard) RightShift(rhs int) Bitboard {
	return Bitboard(C.bitboard(lhs) >> rhs)
}

// Player is a player in a chess game (white or black)
type Player int

// NonePlayer is the abscense of a player
const NonePlayer Player = -1

// White is the white player in the game
const White Player = 0

// Black is the black player in the game
const Black Player = 1

// PieceType is a type of piece in a chess game
type PieceType int

// NonePiece is the abscense of a piece
const NonePiece PieceType = -1

// King is a king piece
const King PieceType = 0

// Pawn is a pawn piece
const Pawn PieceType = 1

// Knight is a knight piece
const Knight PieceType = 2

// Rook is a rook piece
const Rook PieceType = 3

// Bishop is a bishop piece
const Bishop PieceType = 4

// Queen is a queen piece
const Queen PieceType = 5

// Move represents a single player's move that can be made on a chessboard. It represents half a turn -- just one player moving
type Move C.move

// MoveEnd represents the end of MoveGenerator's iteration (ie that MoveGenerator is out of moves)
const MoveEnd Move = Move(0xffffffffffffffff)

// SourceSquare returns the source of the move (ie where the piece being moved starts)
func (move Move) SourceSquare() BoardPos {
	return BoardPos(C.move_source_square(C.move(move)))
}

// DestinationSquare returns the destination of the move (ie where the piece being moved ends up)
func (move Move) DestinationSquare() BoardPos {
	return BoardPos(C.move_destination_square(C.move(move)))
}

// IsPromotion returns true if the move is a pawn promotion
func (move Move) IsPromotion() bool {
	return C.move_is_promotion(C.move(move)) != 0
}

// PromotionPieceType returns what type of piece the pawn is being promoted to, or NonePiece if the move is not a pawn promotion
func (move Move) PromotionPieceType() PieceType {
	return PieceType(int(C.move_promotion_piece(C.move(move))))
}

// IsCapture returns true if the move is a capture
func (move Move) IsCapture() bool {
	return C.move_is_capture(C.move(move)) != 0
}

// CapturePieceType returns what type of piece is being captured, or NonePiece if the move is not a capture
func (move Move) CapturePieceType() PieceType {
	return PieceType(C.move_capture_piece(C.move(move)))
}

// CaptureSquare returns the square that is being captured by the move, or InvalidBoardPos if the move is not a capture
// The capture square is the same as the destination square, except for en passant captures
func (move Move) CaptureSquare() BoardPos {
	return BoardPos(C.move_capture_square(C.move(move)))
}

// IsCastle returns true if the move is a castling, false otherwise.
// If the move is castling, then the source and destination squares are the source and destination of the king (eg white kingside castling is e1 -> g1).
func (move Move) IsCastle() bool {
	return C.move_is_castle(C.move(move)) != 0
}

// ToString converts the move to a string in pure algebraic notation.
// Pure algebraic notation is of the form:
// <src><dst><promote?>
// For example:
// e2e4 -- moving a piece from square e2 to e4
// a7a8q -- moving a pawn from a7 to a8 and promoting to a queen
// Captures don't have a special notation.
// En passant doesn't have a special notation.
func (move Move) ToString() string {
	cs := C.CString("_____")
	defer C.free(unsafe.Pointer(cs))
	C.move_to_str(C.move(move), cs)
	return C.GoString(cs)
}

// MoveFromString constructs a move from a string in pure algebraic notation. board is the board on which the move will be made.
func MoveFromString(str string, board *Board) Move {
	cs := C.CString(str)
	defer C.free(unsafe.Pointer(cs))
	return Move(C.move_from_str(cs, (*C.board)(unsafe.Pointer(board))))
}

// MoveGenerator is an iterator that iterates over all of the legal moves that can be made on a board
type MoveGenerator C.move_gen

// NextMove returns the next legal move that can be made, or returns MoveEnd if the generator is out of moves (exhausted).
func (mg *MoveGenerator) NextMove() Move {
	return Move(C.move_gen_next_move((*C.move_gen)(unsafe.Pointer(mg))))
}

// MakeNextMove returns the next legal move that can be made and applies it to the board, or returns MoveEnd if the generator is out of moves (exhausted).
// You MUST call Board.UnmakeMove with the returned move before calling MakeNextMove or NextMove again.
// MakeNextMove is slightly faster than calling NextMove then Board.MakeMove if you want to evaluate the effect of a move
func (mg *MoveGenerator) MakeNextMove() Move {
	return Move(C.move_gen_make_next_move((*C.move_gen)(unsafe.Pointer(mg))))
}

// IsCheckmate returns true if the player to move is in checkmate.
// You can only call IsCheckmate once the generator has been exhausted (ie once NextMove or MakeNextMove has returned MoveEnd)
func (mg *MoveGenerator) IsCheckmate() bool {
	return C.move_gen_is_checkmate((*C.move_gen)(unsafe.Pointer(mg))) != 0
}

// IsStalemate returns true if the player to move is in stalemate.
// You can only call IsCheckmate once the generator has been exhausted (ie once NextMove or MakeNextMove has returned MoveEnd)
func (mg *MoveGenerator) IsStalemate() bool {
	return C.move_gen_is_stalemate((*C.move_gen)(unsafe.Pointer(mg))) != 0
}

// Board represents the state of a chess board, including the layout of its pieces and the current player to move
type Board C.board

// PieceTypeOnSquare returns the type of piece on the given square, or NonePiece if no piece is on the square
func (b *Board) PieceTypeOnSquare(square BoardPos) PieceType {
	return PieceType(C.board_piece_on_square((*C.board)(unsafe.Pointer(b)), C.board_pos(square)))
}

// PlayerOnSquare returns the player that owns the piece on the given square, or NonePlayer if no piece is on the square
func (b *Board) PlayerOnSquare(square BoardPos) Player {
	return Player(C.board_player_on_square((*C.board)(unsafe.Pointer(b)), C.board_pos(square)))
}

// PlayerBitboard returns a bitboard with bits set for each square that has a piece on it owned by the given player (of any type)
func (b *Board) PlayerBitboard(player Player) Bitboard {
	return Bitboard((*C.board)(unsafe.Pointer(b)).players[int(player)])
}

// PieceBitboard returns a bitboard with bits set for each square that has a piece of the given type on it owned by the given player
func (b *Board) PieceBitboard(player Player, piece PieceType) Bitboard {
	return Bitboard((*C.board)(unsafe.Pointer(b)).players[int(player)] & (*C.board)(unsafe.Pointer(b)).pieces[int(piece)])
}

// PlayerToMove returns the player whose turn it is to move
func (b *Board) PlayerToMove() Player {
	return Player(C.board_player_to_move((*C.board)(unsafe.Pointer(b))))
}

// FullTurnNumber returns the turn number for the board. The turn number starts at 1 and is incremented when a full turn is made (both players make a move)
func (b *Board) FullTurnNumber() int {
    return int(C.board_get_full_turn_number((*C.board)(unsafe.Pointer(b))))
}

// PlayerInCheck returns true if the given player is in check
func (b *Board) PlayerInCheck(player Player) bool {
	return C.board_player_in_check((*C.board)(unsafe.Pointer(b)), C.int(player)) != 0
}

// SquareThreatened returns a bitboard with bits set for each piece of the given player that is threatening the given square.
// IE -- Check if the given square could be taken next turn by attackingPlayer, and return a bitboard with bits set for each of attackingPlayer's pieces that could take the given square
func (b *Board) SquareThreatened(square BoardPos, attackingPlayer Player) Bitboard {
	return Bitboard(C.board_is_square_attacked((*C.board)(unsafe.Pointer(b)), C.board_pos(square), C.int(attackingPlayer)))
}

// MakeMove applies the given move to the board. This modifies the board.
func (b *Board) MakeMove(move Move) {
	C.board_make_move((*C.board)(unsafe.Pointer(b)), C.move(move))
}

// UnmakeMove unapplies the given move. The move being unmade has to be the most recently applied move to the board. This modifies the board
func (b *Board) UnmakeMove(move Move) {
	C.board_unmake_move((*C.board)(unsafe.Pointer(b)), C.move(move))
}

// GetMoveGenerator constructs a move generator for this board.
func (b *Board) GetMoveGenerator() MoveGenerator {
	res := MoveGenerator(C.move_gen{})
	C.move_gen_init((*C.move_gen)(unsafe.Pointer(&res)), (*C.board)(unsafe.Pointer(b)))
	return res
}

// PlayerToMoveIsCheckmate returns true if the player to move is in checkmate.
// This method is slow. If you are iterating through a MoveGenerator, consider using MoveGenerator.IsCheckmate instead.
func (b *Board) PlayerToMoveIsCheckmate() bool {
	return C.board_is_checkmate((*C.board)(unsafe.Pointer(b))) != 0
}

// PlayerToMoveIsStalemate returns true if the player to move is in checkmate.
// This method is slow. If you are iterating through a MoveGenerator, consider using MoveGenerator.IsStalemate instead.
func (b *Board) PlayerToMoveIsStalemate() bool {
	return C.board_is_stalemate((*C.board)(unsafe.Pointer(b))) != 0
}

// Print prints the board on stdout
func (b *Board) Print() {
	C.board_print((*C.board)(unsafe.Pointer(b)))
}

// PrettyPrint prints the board on stdout using ansi escape sequences and unicode chess characters
func (b *Board) PrettyPrint() {
	C.board_print_pretty((*C.board)(unsafe.Pointer(b)))
}

// EnPassantTarget returns the en passant target square, or BoardPosInvalid if there is no en passant target square.
// The en passant target square is the square which a pawn skipped last turn while moving forwards two squares, and is therefore vulnerable to en passant capture. If a pawn didn't move forward two squares last turn, these is no en passant target.
func (b *Board) EnPassantTarget() BoardPos {
	return BoardPos(C.board_get_en_passant_target((*C.board)(unsafe.Pointer(b))))
}

// HasCastlingRights returns true if the given player has castling rights on the given side. side should be King or Queen. Having castling rights does not necessarily mean the player can castle -- rather, it means only they have not moved their king and rook
func (b *Board) HasCastlingRights(player Player, side PieceType) bool {
	return C.board_can_castle((*C.board)(unsafe.Pointer(b)), C.int(player), C.int(side)) != 0
}

// ToString converts the board to a string in Forsyth-Edwards Notation (FEN). The halfmove and turn counters will contain fake values (Board doesn't track them).
func (b *Board) ToString() string {
	// cs contains at least 87 bytes, capable of holding the longest fen string
	cs := C.CString("_______________________________________________________________________________________")
	defer C.free(unsafe.Pointer(cs))
	C.board_to_fen_str((*C.board)(unsafe.Pointer(b)), cs)
	return C.GoString(cs)
}

// BoardFromString constructs a board from a string in Forsyth-Edwards Notation (FEN).
// Because of limitations with go's gc interlopability with C, the memory backing the board has to be manually allocated and deallocated. When you are done with the board, call Board.Free to release it's memory.
// Additionally, due to these limitations, you need to call Board.Copy to make a copy of the board (and you need to call Board.Free once you are done with it). Do not dereference and copy the board pointer returned by the method (your program may segfault).
func BoardFromString(str string) *Board {
	cs := C.CString(str)
	defer C.free(unsafe.Pointer(cs))
	// alloc memory
	res := C.malloc(C.size_t(unsafe.Sizeof(C.board{})))
	C.board_from_fen_str((*C.board)(res), cs)
	return (*Board)(res)
}

// Copy returns a copy of the given board. Because of limitations with go's gc interlopability with C, the memory backing the board has to be manually allocated and deallocated. When you are done with the board, call Board.Free to release it's memory.
func (b *Board) Copy() *Board {
	boardSize := C.size_t(unsafe.Sizeof(C.board{}))
	res := C.malloc(boardSize)
	C.memcpy(unsafe.Pointer(res), unsafe.Pointer(b), boardSize)
	return (*Board)(res)
}

// Free frees the memory backing a Board pointer. You should only call free on pointers returned by BoardFromString or Board.Copy.
// After calling free, don't attempt to access the board. Doing so will cause your program to segfault.
func (b *Board) Free() {
	C.free(unsafe.Pointer(b))
}

// ConnectToServer establishes a connection to the codekata chess server at the given host and port, sends apikey and name, and blocks until the server requests a move.
// When a move is requested, function will be called, which should return the move to make, as well as a map containing debug info (which can be empty)
func ConnectToServer(host string, port string, apikey string, name string, function func(*Board) (Move, map[string]string)) {
	InitLib()
	urlPath := url.URL{Scheme: "ws", Host: fmt.Sprintf("%s:%s", host, port), Path: "/"}
	conn, _, err := websocket.DefaultDialer.Dial(urlPath.String(), nil)
	if err != nil {
		log.Fatal("error connecting to server: ", err)
	}
	defer conn.Close()

	errKey := conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("apikey %s", apikey)))
	if errKey != nil {
		log.Fatal("error sending apikey: ", errKey)
	}
	errName := conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("name %s", name)))
	if errName != nil {
		log.Fatal("error sending name: ", errName)
	}

	for {
		_, message, err := conn.ReadMessage()
		if err != nil {
			log.Fatal("error read message:", err)
		}
		if strings.HasPrefix(string(message), "position") {
			boardStr := string(message)[9:]
			board := BoardFromString(boardStr)
			move, debugInfo := function(board)
			board.Free()

			// send move
			errSend := conn.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf("move %s", move.ToString())))
			if errSend != nil {
				log.Fatal("error sending move: ", errSend)
			}
			// send debug info
			debugStr := "info "
			for key, value := range debugInfo {
			    debugStr += key + " " + value + "`"
			}
			errInfo := conn.WriteMessage(websocket.TextMessage, []byte(debugStr))
			if errInfo != nil {
			    log.Fatal("error sending info: ", errInfo)
			}
		}
	}
}

// InitLib runs pre calculations needed by the C library
func InitLib() {
	C.move_gen_pregenerate()
}
