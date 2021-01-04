#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "chess-util.h"

void __attribute__ ((constructor)) setup() {
  move_gen_pregenerate();
  printf("move_gen_pregenerate done\n");
}

char * clib_board_make_move(char *fen, char *move_str) {
  board board;
  board_from_fen_str(&board, fen);
  move move = move_from_str(move_str, &board);
  board_make_move(&board, move);
  // TODO: don't leak
  char *res = malloc(83);
  board_to_fen_str(&board, res);
  return res;
}

char * clib_board_legal_moves(char *fen) {
  // TODO: don't leak
  int size = 0;
  int cap = 64;
  char *res = malloc(cap);

  board board;
  board_from_fen_str(&board, fen);
  move_gen gen;
  move_gen_init(&gen, &board);
  move move;
  while((move = move_gen_next_move(&gen)) != MOVE_END) {
    char move_str[6];
    move_to_str(move, move_str);
    // append to res
    int size_to_add = strlen(move_str) + 1;
    if (size + size_to_add > cap) {
      cap *= 2;
      res = realloc(res, cap);
    }
    memcpy(res + size, move_str, size_to_add - 1);
    size += size_to_add - 1;
    res[size++] = ',';
  }
  res[size] = '\0';
  return res;
} 

char * gen_fixed_board() {
  return "1nbqkbnr/Pp1ppppp/8/3p4/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 1 2";
}