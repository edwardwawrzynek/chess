use crate::apikey::ApiKey;
use crate::cmd::{ClientCommand, ServerCommand};
use crate::db::{init_db_pool, DBWrapper, Game, GameTypeMap, PgPool};
use crate::error::Error;
use crate::games::{Fmt, GameState, GameTurn};
use crate::models::{GameId, GamePlayer, User, UserId};
use futures_channel::mpsc;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::future::Future;
use std::sync::MutexGuard;
use std::{
    collections::HashMap,
    collections::HashSet,
    hash::Hash,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

/// Topics that a client is interested in receiving messages about
#[derive(PartialEq, Eq, Hash, Debug)]
enum Topic {
    /// Messages for all clients
    Global,
    /// Messages for all clients logged in as a particular user
    UserPrivate(UserId),
    /// Messages about a particular user
    User(UserId),
    /// Messages about a particular game
    Game(GameId),
}

impl Default for Topic {
    fn default() -> Self {
        Topic::Global
    }
}

type ClientTxChannel = mpsc::UnboundedSender<Message>;

/// A collection of connected clients. PeerMap contains a mapping of topics to clients addresses, and client addresses to a communication channel.
#[derive(Debug, Default)]
struct ClientMap {
    // map client -> client transmit channel
    channels: HashMap<SocketAddr, ClientTxChannel>,
    // map topic -> interested clients
    topics: HashMap<Topic, HashSet<SocketAddr>>,
    // map client -> logged in user
    users: HashMap<SocketAddr, UserId>,
}

type ClientMapLock = Arc<Mutex<ClientMap>>;

impl ClientMap {
    /// Insert a client connection
    pub fn insert_client(&mut self, client: SocketAddr, tx: ClientTxChannel) {
        self.channels.insert(client, tx);
    }

    /// Add a client to a topic, creating that topic if it doesn't exist.
    /// In order to register for UserPrivate topics, add_as_user must be used instead.
    pub fn add_to_topic(&mut self, topic: Topic, client: SocketAddr) {
        // clients must register on private topics through add_as_user (to lessen accidental registration for private messages)
        if let Topic::UserPrivate(_) = topic {
            return;
        }
        let topic_map = self.topics.entry(topic).or_insert(HashSet::new());
        topic_map.insert(client);
    }

    /// Check if a client is registered as a logged in user
    pub fn is_user(&self, client: &SocketAddr) -> Option<UserId> {
        self.users.get(client).map(|u| *u)
    }

    /// Unregister a client as a user
    pub fn remove_as_user(&mut self, client: &SocketAddr) {
        if let Some(old_user) = self.is_user(&client) {
            self.remove_from_topic(Topic::UserPrivate(old_user), client);
        }
        self.users.remove(client);
    }

    /// Register a client as a user and add them to the UserPrivate topic for that user.
    /// If the client had been previously registered as a different user, unregister them.
    pub fn add_as_user(&mut self, user_id: UserId, client: SocketAddr) {
        self.remove_as_user(&client);
        let topic_map = self
            .topics
            .entry(Topic::UserPrivate(user_id))
            .or_insert(HashSet::new());
        topic_map.insert(client);
        self.users.insert(client, user_id);
    }

    /// Remove a client from a topic (if the client is in that topic)
    pub fn remove_from_topic(&mut self, topic: Topic, client: &SocketAddr) {
        let topic_map = self.topics.get_mut(&topic);
        if let Some(topic_map) = topic_map {
            topic_map.remove(client);
        }
    }

    /// Remove a client connection
    pub fn remove_client(&mut self, client: &SocketAddr) {
        self.channels.remove(client);
        for (_, topic) in &mut self.topics {
            topic.remove(client);
        }
    }

    /// Send a message to a client
    pub fn send(&self, client: &SocketAddr, msg: Message) -> Result<(), Error> {
        let tx = self.channels.get(client);
        match tx {
            Some(tx) => {
                tx.unbounded_send(msg).unwrap_or_else(|e| {
                    eprintln!(
                        "Can't send message to client -- receiving channel was closed, {}",
                        e
                    )
                });
                Ok(())
            }
            None => Err(Error::NoSuchConnectedClient),
        }
    }

    /// Send a message to all clients in a topic
    pub fn publish(&self, topic: Topic, msg: &Message) -> Result<(), Error> {
        let topic_map = self.topics.get(&topic);
        if let Some(topic_map) = topic_map {
            for client in topic_map {
                self.send(client, msg.clone())?;
            }
        }

        Ok(())
    }
}

/// Convert a game and its players to a game command
fn serialize_game_state(game: &Game, players: &Vec<GamePlayer>) -> ServerCommand {
    let (finished, winner, state) = match &game.instance {
        &None => (false, GameState::InProgress, None),
        Some(inst) => {
            let state = format!("{}", Fmt(|f| inst.serialize(f)));
            match inst.turn() {
                GameTurn::Finished => (
                    true,
                    inst.end_state().unwrap_or(GameState::InProgress),
                    Some(state),
                ),
                _ => (false, GameState::InProgress, Some(state)),
            }
        }
    };
    ServerCommand::Game {
        id: game.id,
        game_type: game.game_type.clone(),
        owner: game.owner_id,
        started: game.instance.is_some(),
        finished,
        winner,
        players: players
            .iter()
            .map(|p| (p.user_id, p.score))
            .collect::<Vec<(UserId, Option<f64>)>>(),
        state,
    }
}

/// Apply a command sent by a client and return a response (if necessary)
fn handle_cmd(
    cmd: &ClientCommand,
    client_map: &Mutex<ClientMap>,
    client_addr: &SocketAddr,
    db_pool: &PgPool,
    game_type_map: &GameTypeMap,
) -> Result<Option<ServerCommand>, Error> {
    use ClientCommand::*;

    // lock the client map
    let clients = || client_map.lock().unwrap();
    // callback when a game's state changes
    let game_update = |game: &Game, players: &Vec<GamePlayer>| {
        // TODO: send board to player whose turn it is
        let cmd = serialize_game_state(game, players);
        clients()
            .publish(Topic::Game(game.id), &Message::from(cmd.to_string()))
            .unwrap_or_else(|e| eprintln!("Can't send game state to client, {}", e));
    };
    // get a database connection
    let db = || DBWrapper::from_pg_pool(db_pool, game_type_map, game_update);
    // load the current user
    fn user<F: Fn(&Game, &Vec<GamePlayer>)>(
        db: &DBWrapper<F>,
        client_addr: &SocketAddr,
        clients: MutexGuard<ClientMap>,
    ) -> Result<User, Error> {
        if let Some(user_id) = clients.is_user(client_addr) {
            db.find_user(user_id)
        } else {
            Err(Error::NotLoggedIn)
        }
    }

    match cmd {
        // --- User Authentication ---
        NewUser {
            name,
            email,
            password,
        } => {
            let user = db()?.new_user(*name, *email, *password)?;
            clients().add_as_user(user.id, *client_addr);
            Ok(None)
        }
        NewTmpUser { name } => {
            let user = db()?.new_tmp_user(*name)?;
            clients().add_as_user(user.id, *client_addr);
            Ok(None)
        }
        Apikey(key) => {
            let user = db()?.find_user_by_apikey(key)?;
            clients().add_as_user(user.id, *client_addr);
            Ok(None)
        }
        Login { email, password } => {
            let user = db()?.find_user_by_credentials(*email, *password)?;
            clients().add_as_user(user.id, *client_addr);
            Ok(None)
        }
        Logout => {
            clients().remove_as_user(client_addr);
            Ok(None)
        }
        // --- User Info / Edit ---
        Name(name) => {
            let db = db()?;
            db.save_user(&User {
                name: name.to_string(),
                ..user(&db, client_addr, clients())?
            })?;
            Ok(None)
        }
        Password(pass) => {
            let db = db()?;
            let hashed = bcrypt::hash(pass, bcrypt::DEFAULT_COST)?;
            db.save_user(&User {
                password_hash: Some(hashed),
                ..user(&db, client_addr, clients())?
            })?;
            Ok(None)
        }
        GenApikey => {
            let db = db()?;
            let key = ApiKey::new();
            db.save_user(&User {
                api_key_hash: key.hash().to_string(),
                ..user(&db, client_addr, clients())?
            })?;
            Ok(Some(ServerCommand::GenApikey(key)))
        }
        SelfUserInfo => {
            let user = user(&db()?, client_addr, clients())?;
            Ok(Some(ServerCommand::SelfUserInfo {
                id: user.id,
                name: user.name,
                email: user.email,
            }))
        }
        // --- Game Creation / Management --
        NewGame(game_type) => {
            let db = &db()?;
            let user = user(db, client_addr, clients())?;
            let game = db.new_game(*game_type, user.id)?;
            Ok(Some(ServerCommand::NewGame(game.id)))
        }
        ObserveGame(game_id) => {
            let (game, players) = db()?.find_game(*game_id)?;
            clients().add_to_topic(Topic::Game(*game_id), *client_addr);
            Ok(Some(serialize_game_state(&game, &players)))
        }
        StopObserveGame(game_id) => {
            clients().remove_from_topic(Topic::Game(*game_id), client_addr);
            Ok(None)
        }
        JoinGame(game_id) => {
            let db = &db()?;
            db.join_game(*game_id, user(db, client_addr, clients())?.id)?;
            Ok(None)
        }
        LeaveGame(game_id) => {
            let db = &db()?;
            db.leave_game(*game_id, user(db, client_addr, clients())?.id)?;
            Ok(None)
        }
        StartGame(game_id) => {
            let db = &db()?;
            db.start_game(*game_id, user(db, client_addr, clients())?.id)?;
            Ok(None)
        }
    }
}

/// Parse a message sent by a client, perform the necessary action, and send any needed response back
fn handle_message(
    msg: &Message,
    client_map: &Mutex<ClientMap>,
    client_addr: &SocketAddr,
    db_pool: &PgPool,
    game_type_map: &GameTypeMap,
) {
    // reply to ping messages
    let reply = if msg.is_close() || msg.is_ping() {
        Ok(None)
    } else {
        // parse the message
        match msg.to_text() {
            Err(_) => Err(Error::MessageParseError),
            Ok(txt) => {
                let cmd = ClientCommand::deserialize(txt);
                match cmd {
                    Ok(cmd) => handle_cmd(&cmd, client_map, client_addr, db_pool, game_type_map),
                    Err(e) => Err(e),
                }
            }
        }
    };

    let reply = reply
        .unwrap_or_else(|e| Some(ServerCommand::Error(e)))
        .unwrap_or(ServerCommand::Okay);

    client_map
        .lock()
        .unwrap()
        .send(client_addr, Message::from(reply.to_string()))
        .unwrap_or_else(|e| eprintln!("Error sending message to client, {}", e));
}

async fn handle_connection(
    client_map: ClientMapLock,
    raw_stream: TcpStream,
    addr: SocketAddr,
    db_pool: Arc<PgPool>,
    game_type_map: Arc<GameTypeMap>,
) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // create channel for sending messages to websocket
    let (tx, rx) = mpsc::unbounded();
    client_map.lock().unwrap().insert_client(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let handle_incoming = incoming.try_for_each(|msg| {
        handle_message(&msg, &*client_map, &addr, &db_pool, &game_type_map);

        future::ok(())
    });

    let send_outgoing = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, send_outgoing);
    future::select(handle_incoming, send_outgoing).await;

    println!("Websocket connection closed: {}", &addr);
    client_map.lock().unwrap().remove_client(&addr);
}

pub fn run_server<'a>(
    url: &'a str,
    db_url: &'a str,
    game_type_map: Arc<GameTypeMap>,
) -> impl Future<Output = ()> + 'a {
    async move {
        // Create application state
        let clients = Arc::new(Mutex::new(ClientMap::default()));
        let db_pool = Arc::new(init_db_pool(db_url).expect("Can't open database"));

        // Setup a tcp server and accept connections
        let try_socket = TcpListener::bind(url).await;
        let listener = try_socket.expect("Failed to bind to port");
        println!("Listening on: {}", url);
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(
                clients.clone(),
                stream,
                addr,
                db_pool.clone(),
                game_type_map.clone(),
            ));
        }
    }
}
