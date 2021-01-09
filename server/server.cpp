#include <random>
#include <sstream>
#include <string>
#include <string_view>
#include <cstdlib>
#include <uWebSockets/App.h>

#include "AsyncFileReader.hpp"
#include "AsyncFileStreamer.hpp"
#include "Middleware.hpp"
#include "server.hpp"

GameInfo::GameInfo(): eval(), data() {}

void GameInfo::serialize(std::ostream &out) {
  out << "eval";
  for(auto score: eval) {
    out << " " << score;
  }

  for(auto& entry: data) {
    out << "`" << entry.first << " " << entry.second;
  }
}

void GameInfo::deserialize_apply(const std::string &in, size_t eval_turn) {
  size_t i = 0;
  while(i < in.size()) {
    std::string key;
    std::string value;

    while(i < in.size() && isspace(in[i]) && in[i] != '`') {
      i++;
    }
    // read key
    while(i < in.size() && !isspace(in[i]) && in[i] != '`') {
      key.push_back(in[i++]);
    }
    // skip space
    while(i < in.size() && isspace(in[i]) && in[i] != '`') {
      i++;
    }
    // read value
    while(i < in.size() && in[i] != '`') {
      value.push_back(in[i++]);
    }

    if(key == "eval") {
      grow_eval_to(eval_turn);
      try {
        eval[eval_turn - 1] = std::stod(value);
      } catch (std::invalid_argument& e) {
        eval[eval_turn - 1] = 0.0;
      }
    } else {
      data[key] = value;
    }
    while(i < in.size() && in[i] == '`') {
      i++;
    }
  }
}

void GameInfo::grow_eval_to(size_t size) {
  while(eval.size() < size) {
    eval.push_back(0);
  }
}

Game::Game(Player *white, Player *black, int id)
    : id(id), white(white), black(black), board_value(), moves(), finished(false),
      score(0) {
  board_from_fen_str(
      &board_value, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}

void Game::make_move(move move) {
  moves.push_back(move);
  board_make_move(&board_value, move);
  // check for end
  bool checkmate = board_is_checkmate(&board_value);
  bool stalemate = board_is_stalemate(&board_value);
  if (checkmate || stalemate) {
    finished = true;
    if (stalemate) {
      score = 0;
    } else {
      score = board_player_to_move(&board_value) == WHITE ? -1 : 1;
    }
  }
}

int Game::adjust_score_for(Player *player) {
  if (player == white) {
    return score;
  } else if (player == black) {
    return -score;
  } else {
    assert(0);
    return 0;
  }
}

void Game::serialize(std::ostream &out) {
  char board_str[87];
  board_to_fen_str(&board_value, board_str);
  out << "game " << id << ", " << white->id << ", " << black->id << ", "
      << board_str << ",";
  for (move move : moves) {
    char move_str[5];
    move_to_str(move, move_str);
    out << " " << move_str;
  }
  out << ", " << finished;
  out << ", " << score;
  out << ", ";
  client_info[0].serialize(out);
  out << ", ";
  client_info[1].serialize(out);
  out << "\n";
}

Player *Game::player_to_move() {
  if (finished) {
    return nullptr;
  } else if (board_player_to_move(&board_value) == WHITE) {
    return white;
  } else {
    return black;
  }
}

int Game::turn_num_before_last_move() {
  int prev_move = board_get_full_turn_number(&board_value);
  if(board_player_to_move(&board_value) == WHITE) {
    return prev_move - 1;
  } else {
    return prev_move;
  }
}

Player::Player(std::string name, int id)
    : id(id), name(std::move(name)), games(), cur_game_index(0), wins(0),
      losses(0), ties(0) {}

Game *Player::get_cur_game() {
  if (cur_game_index >= games.size()) {
    return nullptr;
  } else {
    return games[cur_game_index];
  }
}

void Player::add_game(Game *game) {
  games.push_back(game);
  update_cur_game();
}

void Player::update_cur_game() {
  if (cur_game_index >= games.size()) {
    return;
  } else {
    // check if current game is ended
    Game *cur_game = games[cur_game_index];
    if (cur_game->finished) {
      int game_score = cur_game->adjust_score_for(this);
      if (game_score == 0) {
        ties++;
      } else if (game_score == 1) {
        wins++;
      } else if (game_score == -1) {
        losses++;
      }
      cur_game_index++;
    }
  }
}

void Player::serialize(std::ostream &out, const std::string &api_key,
                       bool do_api_key) {
  out << "player " << id << ", ";
  if (do_api_key) {
    out << api_key << ", ";
  } else {
    out << "-, ";
  }
  out << name << ", " << wins << ", " << losses << ", " << ties << ",";
  for (Game *game : games) {
    out << " " << game->id;
  }
  out << ", " << cur_game_index << "\n";
}

std::pair<std::pair<Player *, std::string>, std::pair<Player *, std::string>>
State::new_game() {
  auto key0 = get_apikey();
  auto key1 = get_apikey();

  auto player0 = std::make_unique<Player>(
      "Unnamed " + get_apikey().substr(0, 4), player_id++);
  auto player0_ptr = player0.get();
  auto player1 = std::make_unique<Player>(
      "Unnamed " + get_apikey().substr(0, 4), player_id++);
  auto player1_ptr = player1.get();
  auto game = std::make_unique<Game>(player0_ptr, player1_ptr, games.size());
  player0->add_game(game.get());
  player1->add_game(game.get());

  games.push_back(std::move(game));
  players[key0] = std::move(player0);
  players[key1] = std::move(player1);

  return std::pair(std::pair(player0_ptr, key0), std::pair(player1_ptr, key1));
}

void State::serialize(std::ostream &out, bool do_api_keys) {
  for (auto &player : players) {
    player.second->serialize(out, player.first, do_api_keys);
  }

  for (auto &game : games) {
    game->serialize(out);
  }
}

State::State() : player_id(0), games(), players() {}

void State::update_cur_games() {
  for (auto &player : players) {
    player.second->update_cur_game();
  }
}

std::string get_apikey() {
  static std::random_device dev;
  static std::mt19937 rng(dev());

  std::uniform_int_distribution<int> dist(0, 15);

  const char *v = "0123456789abcdef";

  std::string res;
  for (int i = 0; i < 8; i++) {
    res += v[dist(rng)];
    res += v[dist(rng)];
  }
  return res;
}

// data attached to each websocket connection
struct PerSocketData {
  // null if observing, active player otherwise
  Player *player;

  PerSocketData() : player(nullptr){};
};

// split a received command into components
std::vector<std::string> parseCommand(std::string_view cmd) {
  std::vector<std::string> res;
  // parse first command part
  std::string first;
  size_t i = 0;
  while (i < cmd.length() && !isspace(cmd[i]) && cmd[i] != '\n' &&
         cmd[i] != '\r') {
    first.push_back(cmd[i]);
    i++;
  }
  res.push_back(std::move(first));
  while (isspace(cmd[i])) {
    i++;
  }
  while (i < cmd.length() && cmd[i] != '\n' && cmd[i] != '\r') {
    std::string cur;
    while (i < cmd.length() && cmd[i] != ',' && cmd[i] != '\n' &&
           cmd[i] != '\r') {
      cur.push_back(cmd[i]);
      i++;
    }
    res.push_back(cur);
    i++;
    while (i < cmd.length() && isspace(cmd[i]) && cmd[i] != '\n' &&
           cmd[i] != '\r') {
      i++;
    }
  }

  return res;
}

auto state = State();

void send_player_cur_position(Player *player, uWS::WebSocket<false, true> *ws) {
  Game *game = player->get_cur_game();
  if (game != nullptr) {
    if (game->player_to_move() == player) {
      std::ostringstream stream;
      char board_str[87];
      board_to_fen_str(&game->board_value, board_str);
      stream << "position " << board_str << "\n";
      ws->publish("player" + std::to_string(player->id), stream.str());
    }
  }
}

int main(int argc, char *argv[]) {
  move_gen_pregenerate();

  if(argc != 2) {
    std::cerr << "usage: " << argv[0] << " http_root" << std::endl;
    return 1;
  }
  AsyncFileStreamer asyncFileStreamer(argv[1]);

  auto port = 9001;
  if(std::getenv("PORT") != nullptr) {
    port = atoi(std::getenv("PORT"));
  }

  /* Keep in mind that uWS::SSLApp({options}) is the same as uWS::App() when
   * compiled without SSL support.
   * You may swap to using uWS:App() if you don't need SSL */
  uWS::App()
      .ws<PerSocketData>(
          "/*",
          {/* Settings */
           .compression = uWS::SHARED_COMPRESSOR,
           .maxPayloadLength = 16 * 1024,
           .idleTimeout = 0,
           .maxBackpressure = 1 * 1024 * 1024,
           /* Handlers */
           .upgrade = nullptr,
           .open =
               [](auto * /*ws*/) {
                 /* Open event here, you may access ws->getUserData() which
                  * points to a PerSocketData struct */
               },
           .message =
               [](auto *ws, std::string_view message, uWS::OpCode opCode) {
                 std::vector<std::string> cmd = parseCommand(message);
                 // command: apikey key
                 // register as a player with the given key
                 if (cmd[0] == "apikey") {
                   if (cmd.size() < 2) {
                     ws->send("error expected 1 argument to command apikey",
                              opCode);
                   } else if (state.players.contains(cmd[1])) {
                     Player *player = state.players[cmd[1]].get();
                     ((PerSocketData *)ws->getUserData())->player = player;
                     ws->subscribe("player" + std::to_string(player->id));
                     // send position if player is up
                     send_player_cur_position(player, ws);
                   } else {
                     ws->send("error invalid api key " + cmd[1], opCode);
                   }
                 }
                 // command: observe
                 // register as a game observer, and get sent the current state
                 // of the game
                 else if (cmd[0] == "observe") {
                   ws->subscribe("observe");
                   // send current state
                   std::ostringstream stream;
                   state.serialize(stream, false);
                   if (!stream.str().empty()) {
                     ws->send(stream.str(), opCode, true);
                   }
                 }
                 // command: name name
                 // set the current player's name
                 else if (cmd[0] == "name") {
                   auto player = ((PerSocketData *)ws->getUserData())->player;
                   if (cmd.size() < 2) {
                     ws->send("error expected 1 argument to command name",
                              opCode);
                   } else if (player == nullptr) {
                     ws->send("error not registered as a player (use apikey "
                              "command)",
                              opCode);
                   } else {
                     player->name = cmd[1];
                     std::ostringstream stream;
                     player->serialize(stream, "", false);
                     ws->publish("observe", stream.str());
                   }
                 }
                 // command: newgame
                 // create a new game and return api keys
                 else if (cmd[0] == "newgame") {
                   auto res = state.new_game();
                   state.update_cur_games();
                   Game *game = res.first.first->get_cur_game();
                   // send api keys back
                   ws->send("newgame " + std::to_string(game->id) + ", " +
                                std::to_string(res.first.first->id) + ", " +
                                res.first.second + ", " +
                                std::to_string(res.second.first->id) + ", " +
                                res.second.second + "\n",
                            opCode);
                   // publish players + game
                   std::ostringstream stream;
                   res.first.first->serialize(stream, "", false);
                   res.second.first->serialize(stream, "", false);
                   game->serialize(stream);
                   ws->publish("observe", stream.str());
                 }
                 // command: move move_str
                 // make the move on the last sent position
                 else if (cmd[0] == "move") {
                   auto player = ((PerSocketData *)ws->getUserData())->player;
                   if (cmd.size() < 2) {
                     ws->send("error expected 1 argument to command move",
                              opCode);
                   } else if (player == nullptr) {
                     ws->send("error not registered as a player (use apikey "
                              "command)",
                              opCode);
                   } else {
                     // make sure it is this player's turn
                     Game *game = player->get_cur_game();
                     if (game == nullptr || game->player_to_move() != player) {
                       ws->send("error not player's turn", opCode);
                     } else {
                       // make sure move is legal
                       move move = move_from_str(cmd[1].c_str(), &game->board_value);
                       if (!move_is_legal(move, &game->board_value)) {
                         ws->send("error move is illegal", opCode);
                       } else {
                         game->make_move(move);
                         state.update_cur_games();
                         game->client_info[0].grow_eval_to(game->turn_num_before_last_move());
                         game->client_info[1].grow_eval_to(game->turn_num_before_last_move());
                         // send new game positions
                         send_player_cur_position(game->white, ws);
                         send_player_cur_position(game->black, ws);
                         // broadcast changes
                         std::ostringstream stream;
                         game->white->serialize(stream, "", false);
                         game->black->serialize(stream, "", false);
                         game->serialize(stream);
                         ws->publish("observe", stream.str());
                       }
                     }
                   }
                 }
                 // command: info info_str
                 // send client info to server for current player
                 else if(cmd[0] == "info") {
                   auto player = ((PerSocketData *)ws->getUserData())->player;
                   if (player == nullptr) {
                     ws->send("error not registered as a player (use apikey "
                              "command)",
                              opCode);
                   } else if (cmd.size() < 2) {
                     ws->send("error expected 1 argument to command info",
                              opCode);
                   } else {
                     Game *game = player->get_cur_game();
                     if (game == nullptr) {
                       ws->send("error player has no active games", opCode);
                     } else {
                       int player_index = player == game->white ? 0 : 1;
                       game->client_info[player_index].deserialize_apply(cmd[1], game->turn_num_before_last_move());
                       // broadcast changes
                       std::ostringstream stream;
                       game->serialize(stream);
                       ws->publish("observe", stream.str());
                     }
                   }
                 }
                 // command: playerid
                 // return the player id associated with the current api key
                 else if (cmd[0] == "playerid") {
                   auto player = ((PerSocketData *)ws->getUserData())->player;
                   if (player == nullptr) {
                     ws->send("error not registered as a player (use apikey "
                              "command)",
                              opCode);
                   } else {
                     auto id = player->id;
                     ws->send("playerid " + std::to_string(id) + "\n", opCode);
                   }
                 } else {
                   ws->send("error invalid operation " + cmd[0], opCode);
                 }
               },
           .drain =
               [](auto * /*ws*/) {
                 /* Check ws->getBufferedAmount() here */
               },
           .ping =
               [](auto * /*ws*/) {
                 /* Not implemented yet */
               },
           .pong =
               [](auto * /*ws*/) {
                 /* Not implemented yet */
               },
           .close =
               [](auto * /*ws*/, int /*code*/, std::string_view /*message*/) {
                 /* You may access ws->getUserData() here */
               }})
      .get("/*", [&asyncFileStreamer](auto *res, auto *req) {
        // act as http server for frontend
        serveFile(res, req);
        asyncFileStreamer.streamFile(res, req->getUrl());
      }).listen(port,
              [port](auto *listen_socket) {
                if (listen_socket) {
                  std::cout << "Listening on port " << port << std::endl;
                }
              })
      .run();
}
