#ifndef H_SERVER_INCL
#define H_SERVER_INCL

#include <iostream>
#include <map>
#include <vector>
#include <memory>
extern "C" {
#include "chess-util.h"
}

class Player;

class Game {
public:
  int id;
  Player *white, *black;
  // current state of the game
  board board_value;
  // moves previously played
  std::vector<move> moves;
  // if the game is finished
  bool finished;
  // score if game is finished (0 = tie, 1 = white win, -1 = black win)
  int score;

  Game(Player *white, Player *black, int id);

  // make a move, and update board + finished state
  void make_move(move move);

  int adjust_score_for(Player *player);

  Player *player_to_move();

  void serialize(std::ostream &out);
};

class Player {
public:
  int id;
  std::string name;
  std::vector<Game *> games;
  size_t cur_game_index;

  int wins;
  int losses;
  int ties;

  Player(std::string name, int id);

  Game *get_cur_game();
  void add_game(Game *game);
  void update_cur_game();

  void serialize(std::ostream &out, const std::string &api_key,
                 bool do_api_key);
};

class State {
public:
  int player_id;
  std::vector<std::unique_ptr<Game>> games;
  std::map<std::string, std::unique_ptr<Player>> players;

  std::pair<std::pair<Player *, std::string>, std::pair<Player *, std::string>>
  new_game();

  void serialize(std::ostream &out, bool do_api_keys);

  void update_cur_games();

  State();
};

std::string get_apikey();
#endif