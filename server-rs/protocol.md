# Chess Server Protocol

Communication takes place over a full-duplex websocket connection. Client and server send commands in the same format:
```
cmd arg0, arg1, arg2, arg3
```

## Commands
|Command|Sender|Description|
-|-|-
|`error <msg>`|Server|Report an error that occurred in processing a command.|
|`new_user <name>, <email>, <password>`|Create and log in as a new user.|
|`new_tmp_user <name>`|Create and log in as a new user without an email/password|
|`apikey <key>`|Client|Log in with the given api key.|
|`login <email> <password>`|Client|Log in with an email and password.|
|`logout`|Client|Log out of the logged in user's account.|
|`name <name>`|Client|Set the logged in user's name.|
|`password <pass>`|Client|Set the logged in user's password.|
|`gen_apikey`|Client|Re-generate the logged in user's apikey (server responds with `gen_apikey`).|
|`gen_apikey <key>`|Server|Return the user's generated apikey.|
|`self_user_info`|Client|Get information on the logged in user (server responds with `self_user_info`).|
|`self_user_info <id>, <email>, <name>`|Server|Send information about the current user to the client.|


