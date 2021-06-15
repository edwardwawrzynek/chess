# Chess Server Protocol

Communication takes place over a full-duplex websocket connection. Client and server send commands in the same format:
```
cmd arg0, arg1, arg2, arg3
```

## Commands
### User commands
|Command|Sender|Description|
-|-|-
|`error <msg>`|Server|Report an error that occurred in processing a command.|
|`okay`|Server|Report that a command was processed successfully, but no response to the client is needed.|
|`new_user <name>, <email>, <password>`|Client|Create and log in as a new user.|
|`new_tmp_user <name>`|Client|Create and log in as a new user without an email/password|
|`apikey <key>`|Client|Log in with the given api key.|
|`login <email> <password>`|Client|Log in with an email and password.|
|`logout`|Client|Log out of the logged in user's account.|
|`name <name>`|Client|Set the logged in user's name.|
|`password <pass>`|Client|Set the logged in user's password.|
|`gen_apikey`|Client|Re-generate the logged in user's apikey (server responds with `gen_apikey`).|
|`gen_apikey <key>`|Server|Return the user's generated apikey.|
|`self_user_info`|Client|Get information on the logged in user (server responds with `self_user_info`).|
|`self_user_info <id>, <email>, <name>`|Server|Send information about the current user to the client.|
### Game commands
|Command|Sender|Description|
-|-|-
|`new_game <type>`|Client|Create a new game of the given type (server responds with `new_game`).|
|`new_game <id>`|Server|Return the new game's id.|
|`observe_game <id>`|Client|Get the state of the game with the given id, and receive updates when that state changes (server responds with `game`).|
|`stop_observe_game <id>`|Client|Stop receiving updates about the state of the game with the given id.|
|`game <id>,<type>,<owning_user_id>,<started>,<finished>,<winner_id OR "tie">,[[<player0_id>,<player0_score>],...],<game_state OR "-">`|Server|Send a game's state to the client.|
|`join_game <id>`|Client|Join the game with the given id. The game must not be started yet.|
|`leave_game <id>`|Client|Leave the game with the given id. The game must not be started yet.|
|`start_game <id>`|Client|Start the game with the given id. The logged in user must own the game.|
