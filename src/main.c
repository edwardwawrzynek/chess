#include "move_gen.h"
#include "stdlib.h"
#include <stdio.h>
#include <assert.h>

int perft(int depth, int cur, board* board) {
  if(depth == 0)
    return 1;

  move_gen generate;
  move_gen_init(&generate, board);

  int count = 0;
  move move;
  while((move = move_gen_make_next_move(&generate)) != MOVE_END) {
    int moves = perft(depth - 1, cur + 1, board);
    if(cur == 0) {
      char move_str[6];
      move_to_str(move, move_str);
      char board_fen[87];
      board_to_fen_str(board, board_fen);
      printf("%s: %i\t(%s)\n", move_str, moves, board_fen);
    }
    count += moves;
    move_gen_unmake_move(board, move);
  }
  return count;
}

int main(int argc, char **argv) {
  assert(argc == 3);
  move_gen_pregenerate();
  board game;
  board_from_fen_str(&game, argv[1]);
  board_print(&game);
  int count = perft(strtol(argv[2], NULL, 10), 0, &game);
  printf("total %i\n", count);

  return 0;
}

/* int main(int argc, char **argv) {
  move_gen_pregenerate();
  board game;
  board_from_fen_str(&game, "rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
  board_print(&game);
  move_gen gen;
  move_gen_init(&gen, &game, 0);
  move move;
  while((move = move_gen_next_move(&gen)) != MOVE_END) {
    char str[6];
    move_to_str(move, str);
    printf("%s is_cap: %i\n", str, move_is_capture(move));
  }
} */
