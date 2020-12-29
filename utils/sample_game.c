#include "chess-util.h"
#include <stddef.h>
#include <stdio.h>

#define EVAL_INF 10000000
#define EVAL_CHECKMATE 1000000

int max(int a, int b) {
  if (a > b) {
    return a;
  } else {
    return b;
  }
}

int piece_scores[6] = {0, 1, 3, 5, 3, 9};

int eval(board *board) {
  int score = 0;
  for (int player = WHITE; player <= BLACK; player++) {
    int coeff = player == WHITE ? 1 : -1;
    for (int piece = PAWN; piece <= QUEEN; piece++) {
      score += coeff * piece_scores[piece] *
               bitboard_popcount(board->players[player] & board->pieces[piece]);
    }
  }

  return score;
}

int minimax(board *board, move *bestmove, int depth, int alpha, int beta,
            int color) {
  // if depth is 0, run evaluation
  if (depth == 0) {
    return color * eval(board);
  }

  move_gen gen;
  move_gen_init(&gen, board);
  move move;

  int value = -EVAL_INF;

  while ((move = move_gen_make_next_move(&gen)) != MOVE_END) {
    // find score for child node
    int childvalue = -minimax(board, NULL, depth - 1, -beta, -alpha, -color);
    // undo the move
    board_unmake_move(board, move);
    // if that score was best, record it + save it as bestmove
    if (childvalue > value) {
      value = childvalue;
      if (bestmove != NULL) {
        *bestmove = move;
      }
    }
    // adjust alpha
    alpha = max(alpha, value);
    // check for beta cutoff
    if (alpha >= beta) {
      break;
    }
  }

  if (board_is_checkmate(board)) {
    return -EVAL_CHECKMATE;
  } else if (board_is_stalemate(board)) {
    return 0;
  }

  return value;
}

move run(board *board) {
  move bestmove;
  minimax(board, &bestmove, 5, -EVAL_INF, EVAL_INF,
          board_player_to_move(board) == WHITE ? 1 : -1);
  return bestmove;
}