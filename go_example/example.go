package main

import (
	"fmt"
	"os"

	"github.com/edwardwawrzynek/chess/go_chess"
)

/**
 * Sample Go Codekata Chess Client
 *
 * This sample implements a very basic minimax
 */

// EvalInfinity is a value which eval is guaranteed to return less than
const EvalInfinity = 100000000

// evaluate a board from white's perspective
func eval(board *go_chess.Board) int {
	// evaluate based purely on material
	return (
	// white
	1*board.PieceBitboard(go_chess.White, go_chess.Pawn).Count() +
		3*board.PieceBitboard(go_chess.White, go_chess.Knight).Count() +
		3*board.PieceBitboard(go_chess.White, go_chess.Bishop).Count() +
		5*board.PieceBitboard(go_chess.White, go_chess.Rook).Count() +
		9*board.PieceBitboard(go_chess.White, go_chess.Queen).Count() +
		// black
		-1*board.PieceBitboard(go_chess.Black, go_chess.Pawn).Count() +
		-3*board.PieceBitboard(go_chess.Black, go_chess.Knight).Count() +
		-3*board.PieceBitboard(go_chess.Black, go_chess.Bishop).Count() +
		-5*board.PieceBitboard(go_chess.Black, go_chess.Rook).Count() +
		-9*board.PieceBitboard(go_chess.Black, go_chess.Queen).Count())
}

// minimax runs a minimax on the board and return (score, best move)
// color is 1 if white, -1 if black
func minimax(board *go_chess.Board, depth int, color int) (int, go_chess.Move) {
	// if we reached bottom depth, evaluate
	if depth == 0 {
		value := color * eval(board)
		return value, go_chess.MoveEnd
	}

	// otherwise, generate all moves and score them
	moveGenerate := board.GetMoveGenerator()

	// get the first legal move, apply it
	currentMove := moveGenerate.MakeNextMove()
	// current best score and move found so far
	value := -EvalInfinity
	bestMove := currentMove
	// loop through all moves
	for currentMove != go_chess.MoveEnd {
		// MakeNextMove() already made the move for us, so we just visit the board
		childVisit, _ := minimax(board, depth-1, -color)
		// we don't care about the best child move, so just get the score
		// inverted because child score was for opposite color
		childScore := -childVisit

		if childScore > value {
			value = childScore
			bestMove = currentMove
		}

		// unapply the current move
		board.UnmakeMove(currentMove)
		// and make the next legal move
		currentMove = moveGenerate.MakeNextMove()
	}

	// check for checkmate (score -infinity) or stalemate (score 0)
	if moveGenerate.IsCheckmate() {
		value = -EvalInfinity
	} else if moveGenerate.IsStalemate() {
		value = 0
	}

	return value, bestMove
}

func main() {
	args := os.Args

	if len(args) < 5 {
		fmt.Printf("usage: %s host port apikey name\n", args[0])
		fmt.Printf("example: %s codekatachess.herokuapp.com 80 API_KEY Computer\n", args[0])
		os.Exit(1)
	}

	host := args[1]
	port := args[2]
	apikey := args[3]
	name := args[4]

	go_chess.ConnectToServer(host, port, apikey, name, func(b *go_chess.Board) go_chess.Move {
		b.Print()
		color := 1
		if b.PlayerToMove() == go_chess.Black {
			color = -1
		}

		score, move := minimax(b, 4, color)
		fmt.Printf("making move %s, with score %d\n", move.ToString(), score)
		return move
	})
}
