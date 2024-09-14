/*
use std::{collections::{HashMap, HashSet}, sync::{atomic::AtomicUsize, Arc}, time::{Duration, Instant}};

use actix_web::{dev::ServerHandle, rt, web, Either, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use futures_util::{io, StreamExt};
use log::{debug, info, warn};
use rand::{thread_rng, Rng};
use tokio::{pin, sync::{mpsc, oneshot}, time::interval};

/// Sets how often clients are checked for life.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Sets how long clients have before they timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

type ConnId = usize;
type RoomId = String;

#[derive(Debug, Clone)]
enum LoadingResponse {
    /// The progress of an operation (by current object count, total object count)
    Progress(u64, u64),
    /// The String payload.
    TextData(String),
}

impl From<&str> for LoadingResponse {
    fn from(value: &str) -> Self {
        Self::TextData(value.to_string())
    }
}

impl From<String> for LoadingResponse {
    fn from(value: String) -> Self {
        Self::TextData(value)
    }
}


#[derive(Debug)]
enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<LoadingResponse>,
        res_tx: oneshot::Sender<ConnId>,
    },

    Disconnect {
        conn: ConnId,
    },

    List {
        res_tx: oneshot::Sender<Vec<RoomId>>,
    },

    Join {
        conn: ConnId,
        room: RoomId,
        res_tx: oneshot::Sender<()>,
    },

    Message {
        msg: LoadingResponse,
        conn: ConnId,
        res_tx: oneshot::Sender<()>,
    },
}

#[derive(Debug)]
pub struct LoadingServer {
    sessions: HashMap<ConnId, mpsc::UnboundedSender<LoadingResponse>>,
    rooms: HashMap<RoomId, HashSet<ConnId>>,
    visitor_count: Arc<AtomicUsize>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl LoadingServer {
    pub fn new() -> (Self, LoadingServerHandle) {
        let mut rooms = HashMap::with_capacity(4);

        rooms.insert("main".to_owned(), HashSet::new());

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        return (
            Self {
                sessions: HashMap::new(),
                rooms,
                visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            LoadingServerHandle { cmd_tx },
        );
    }

    pub async fn send_system_message(&self, room: &str, skip: ConnId, msg: impl Into<LoadingResponse>) {
        if let Some(sessions) = self.rooms.get(room) {
            let msg: LoadingResponse = msg.into();

            for conn_id in sessions {
                if *conn_id != skip {
                    if let Some(tx) = self.sessions.get(conn_id) {
                        let _ = tx.send(msg.clone());
                    }
                }
            }
        }
    }

    pub async fn send_message(&self, conn: ConnId, msg: impl Into<LoadingResponse>) {
        if let Some(room) = self
            .rooms
            .iter()
            .find_map(|(room, participants)| participants.contains(&conn).then_some(room))
        {
            self.send_system_message(room, conn, msg).await;
        }
    }

    pub async fn connect(&mut self, tx: mpsc::UnboundedSender<LoadingResponse>) -> ConnId {
        info!("Someone Joined!");

        self.send_system_message("main", 0, "Someone Joined!").await;

        let id = thread_rng().gen::<ConnId>();
        self.sessions.insert(id, tx);

        self.rooms.entry("main".to_owned()).or_default().insert(id);

        let count = self.visitor_count.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        self.send_system_message("main", 0, format!("Total Visitors: {count}")).await;

        return id;
    }

    pub async fn disconnect(&mut self, conn_id: ConnId) {
        debug!("Someone Disconnected!");

        let mut rooms: Vec<RoomId> = Vec::new();

        if self.sessions.remove(&conn_id).is_some() {
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&conn_id) {
                    rooms.push(name.to_owned());
                }
            }
        }

        for room in rooms {
            self.send_system_message(&room, 0, "Someone Disconnected!").await;
        }
    }

    fn list_rooms(&mut self) -> Vec<RoomId> {
        self.rooms.keys().cloned().collect()
    }

    pub async fn join_room(&mut self, conn_id: ConnId, room: RoomId) {
        let mut rooms = Vec::new();
        
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&conn_id) {
                rooms.push(n.to_owned());
            }
        }

        for room in rooms {
            self.send_system_message(&room, 0, "Someone Disconnected!").await;
        }

        self.rooms.entry(room.clone()).or_default().insert(conn_id);
        self.send_system_message(&room, conn_id, "Someone Connected!").await;
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn_tx, res_tx } => {
                    let conn_id = self.connect(conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }

                Command::List { res_tx } => {
                    let _ = res_tx.send(self.list_rooms());
                }

                Command::Join { conn, room, res_tx } => {
                    self.join_room(conn, room).await;
                    let _ = res_tx.send(());
                }

                Command::Message { msg, conn, res_tx } => {
                    self.send_message(conn, msg).await;
                    let _ = res_tx.send(());
                }
            }
        }

        Ok(())
    }

}

#[derive(Debug, Clone)]
pub struct LoadingServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}


impl LoadingServerHandle {
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<LoadingResponse>) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Connect { conn_tx, res_tx })
            .unwrap();

        return res_rx.await.unwrap();
    }

    pub async fn list_rooms(&self) -> Vec<RoomId> {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx.send(Command::List { res_tx }).unwrap();

        return res_rx.await.unwrap();
    }

    pub async fn join_room(&self, conn: ConnId, room: impl Into<RoomId>) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Join { conn, room: room.into(), res_tx }).unwrap();

        return res_rx.await.unwrap();
    }

    pub async fn send_message(&self, conn: ConnId, msg: impl Into<LoadingResponse>) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Message { msg: msg.into(), conn, res_tx })
            .unwrap();

        return res_rx.await.unwrap();
    }

    pub async fn disconnect(&self, conn: ConnId) {
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}

pub async fn echo(loading_server: LoadingServer, req: HttpRequest, mut session: actix_ws::Session, msg_stream: actix_ws::MessageStream) -> Result<HttpResponse, Error> {

    debug!("Starting WS Connection...");

    // let mut name = None;
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(CLIENT_TIMEOUT);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    let conn_id = loading_server.connect(conn_tx).await;

    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut msg_stream = pin!(msg_stream);

    let close_reason = loop {
        let tick = pin!(interval.tick());
        let msg_rx = pin!(conn_rx.recv());

        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                debug!("msg: {msg:?}");

                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();

                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        process_text_msg(&chat_server, &mut session, &text, &mut name).await;
                    }

                    AggregatedMessage::Binary(_bin) => {
                        warn!("Unexpected Binary Message!");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }
        }
    }

    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuous frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    session.text(text).await.unwrap();
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
    });

    return Ok(res);
}
*/
