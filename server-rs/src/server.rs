use crate::apikey::ApiKey;
use crate::cmd::{ClientCommand, ServerCommand};
use crate::db::{init_db_pool, DBWrapper, PgPool};
use crate::error::Error;
use crate::models::{User, UserId};
use futures_channel::mpsc;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
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
                tx.unbounded_send(msg)
                    .expect("Receiving channel was closed");
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

/// Apply a command sent by a client and return a response (if necessary)
fn handle_cmd(
    cmd: &ClientCommand,
    client_map: &Mutex<ClientMap>,
    client_addr: &SocketAddr,
    db_pool: &PgPool,
) -> Result<Option<ServerCommand>, Error> {
    use ClientCommand::*;

    // get a database connection
    let db = || DBWrapper::from_pg_pool(db_pool);
    // lock the client map
    let clients = || client_map.lock().unwrap();
    // load the current user
    let user = |db: &DBWrapper| {
        if let Some(user_id) = clients().is_user(client_addr) {
            db.find_user(user_id)
        } else {
            Err(Error::NotLoggedIn)
        }
    };

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
                ..user(&db)?
            })?;
            Ok(None)
        }
        Password(pass) => {
            let db = db()?;
            let hashed = bcrypt::hash(pass, bcrypt::DEFAULT_COST)?;
            db.save_user(&User {
                password_hash: Some(hashed),
                ..user(&db)?
            })?;
            Ok(None)
        }
        GenApikey => {
            let db = db()?;
            let key = ApiKey::new();
            db.save_user(&User {
                api_key_hash: key.hash().to_string(),
                ..user(&db)?
            })?;
            Ok(Some(ServerCommand::GenApikey(key)))
        }
        SelfUserInfo => {
            let user = user(&db()?)?;
            Ok(Some(ServerCommand::SelfUserInfo {
                id: user.id,
                name: user.name,
                email: user.email,
            }))
        }
    }
}

/// Parse a message sent by a client, perform the necessary action, and send any needed response back
fn handle_message(
    msg: &Message,
    client_map: &Mutex<ClientMap>,
    client_addr: &SocketAddr,
    db_pool: &PgPool,
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
                    Ok(cmd) => handle_cmd(&cmd, client_map, client_addr, db_pool),
                    Err(e) => Err(e),
                }
            }
        }
    };

    let reply = reply.unwrap_or_else(|e| Some(ServerCommand::Error(e)));

    if let Some(reply) = reply {
        client_map
            .lock()
            .unwrap()
            .send(client_addr, Message::from(reply.to_string()))
            .expect("Error sending message to client");
    }
}

async fn handle_connection(
    client_map: ClientMapLock,
    raw_stream: TcpStream,
    addr: SocketAddr,
    db_pool: Arc<PgPool>,
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
        handle_message(&msg, &*client_map, &addr, &db_pool);

        future::ok(())
    });

    let send_outgoing = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, send_outgoing);
    future::select(handle_incoming, send_outgoing).await;

    println!("Websocket connection closed: {}", &addr);
    client_map.lock().unwrap().remove_client(&addr);
}

pub async fn run_server(url: &str, db_url: &str) {
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
        ));
    }
}