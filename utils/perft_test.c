#include "chess-util.h"
#include <stdio.h>

int64_t perft(int depth, board* board) {
  if(depth == 0)
    return 1;

  move_gen generate;
  move_gen_init(&generate, board);

  int64_t count = 0;
  move move;
  while((move = move_gen_make_next_move(&generate)) != MOVE_END) {
    count += perft(depth - 1, board);
    move_gen_unmake_move(board, move);
  }
  return count;
}

int run_test(FILE *in) {
  int fail = 0;
  char fen_string[87];
  int depth = 0;
  fscanf(in, "position %[^\n]\n", fen_string);
  board board;
  board_from_fen_str(&board, fen_string);
  fscanf(in, "depth %i\n", &depth);
  for(int i = 1; i <= depth; i++) {
    int64_t expect;
    fscanf(in, "%li\n", &expect);
    if(expect == -1) {
      continue;
    }
    int64_t actual = perft(i, &board);
    if(expect != actual) {
      fprintf(stderr, "pertf_test: error:\non board: %s\nat depth: %i\nperft found %li moves (expected %li)\n", fen_string, i, actual, expect);
      fail = 1;
    } else {
      printf("pertf_test: good: board %s at depth %i good (%li moves found)\n", fen_string, i, actual);
    }
  }
  return fail;
}

int main(int argc, char **argv) {
  move_gen_pregenerate();
  return run_test(stdin);
}
