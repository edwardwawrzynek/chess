# C++ Chess Bindings
The c++ bindings:
- handles interaction with the codekata chess server
- provides classes to represent chess boards, pieces, and moves
- provides very fast legal move generation and application

The major components are:
- `chess::Board`: Represents the state of a chess game
- `chess::BoardPos`: A position (square) on a chess board
- `chess::Player` : Player type (white or black)
- `chess::PieceType`: Piece type (pawn, rook, knight, etc)
- `chess::Bitboard`: Contains a single bit for each square on a chess board. Represents something about a each square (such as where white has pawns).
- `chess::Move`: A single player's move that can be applied to a board
- `chess::MoveGenerator`: An iterator over all the legal moves that can be played on a board
- `chess::connectToServer()`: A function that connects to a codekata chess server, and calls the passed function whenever a move needs to be made


