#include "chess-util.h"

#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <wsclient/wsclient.h>

move run(board *board);

/* ----------- API Interface ------------- */

char *apikey;
char *name;

int onclose(wsclient *c) {
  fprintf(stderr, "onclose called: %d\n", c->sockfd);
  return 0;
}

int onerror(wsclient *c, wsclient_error *err) {
  fprintf(stderr, "onerror: (%d): %s\n", err->code, err->str);
  if (err->extra_code) {
    errno = err->extra_code;
    perror("recv");
  }
  return 0;
}

int onmessage(wsclient *c, wsclient_message *msg) {
  printf("onmessage: (%llu): %s\n", msg->payload_len, msg->payload);
  if (strlen(msg->payload) >= 9 && !memcmp(msg->payload, "position", 8)) {
    char *board_str = msg->payload + 9;
    board game;
    board_from_fen_str(&game, board_str);
    printf("solving board: \n");
    board_print(&game);
    move move = run(&game);
    char move_str[6];
    move_to_str(move, move_str);
    printf("making move: %s\n", move_str);
    char *move_msg = malloc(6 + strlen(move_str));
    snprintf(move_msg, 6 + strlen(move_str), "move %s", move_str);
    libwsclient_send(c, move_msg);
    free(move_msg);
  }
  return 0;
}

int onopen(wsclient *c) {
  char *key_msg = malloc(8 + strlen(apikey));
  snprintf(key_msg, 8 + strlen(apikey), "apikey %s", apikey);
  libwsclient_send(c, key_msg);
  char *name_msg = malloc(6 + strlen(name));
  snprintf(name_msg, 6 + strlen(name), "name %s", name);
  libwsclient_send(c, name_msg);

  free(key_msg);
  free(name_msg);
  return 0;
}

int main(int argc, char **argv) {
  if (argc != 3) {
    printf("usage: %s apikey name\n", argv[0]);
    return 1;
  }
  apikey = argv[1];
  name = argv[2];

  move_gen_pregenerate();

  // Initialize new wsclient * using specified URI
  wsclient *client = libwsclient_new("ws://localhost:9001");
  if (!client) {
    fprintf(stderr, "Unable to initialize new WS client.\n");
    exit(1);
  }
  // set callback functions for this client
  libwsclient_onopen(client, &onopen);
  libwsclient_onmessage(client, &onmessage);
  libwsclient_onerror(client, &onerror);
  libwsclient_onclose(client, &onclose);
  // starts run thread.
  libwsclient_run(client);
  sleep(1);
  // blocks until run thread for client is done.
  libwsclient_finish(client);
  return 0;
}
