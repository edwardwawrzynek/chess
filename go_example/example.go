package main

import (
	"fmt"

	"github.com/edwardwawrzynek/chess/go_chess"
)

func main() {
	go_chess.ConnectToServer("localhost", "9001", "88c896d7da5fb8c4", "Computer", func(b *go_chess.Board) go_chess.Move {
		b.Print()
		fmt.Printf(b.ToString())

		gen := b.GetMoveGenerator()
		return gen.NextMove()
	})
}
