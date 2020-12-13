#include "move_gen.h"
#include "stdlib.h"
#include <stdio.h>

int perft(int depth, board* board) {
  if(depth == 0)
    return 1;

  move_gen generate;
  move_gen_init(&generate, board, 0);

  int count = 0;
  move move;
  while((move = move_gen_make_next_move(&generate)) != MOVE_END) {
    count += perft(depth - 1, board);
    move_gen_unmake_move(board, move);
  }
  return count;
}

int main(int argc, char **argv) {
  move_gen_pregenerate();
  board game;
  board_from_fen_str(&game, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
  board_print(&game);
  int count = perft(4, &game);
  printf("%i\n", count);

  return 0;
}
