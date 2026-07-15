use tokio::{ // async (non-blocking) runtime 4 manage tasks and io ops
    net::{TcpListener, TcpStream},// stream- broadcast messages
    sync::broadcast,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},// async read/write to stream
};



use serde::{Serialize, Deserialize};
use chrono::Local; // 4 working w local date/time
use std::error::Error;
#[derive(Debug, Clone, Serialize, Deserialize)]
/// an attr instructing the compile to generate impl for the traits above

struct ChatMessage {
    username: String,
    content: String,
    timestamp: String,
    message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum MessageType {
    Join,
    Leave,
    UserMessage,
}


#[tokio::main]
    /// macro that convert fn main 4rm sync to async using tokio
    /// creates a tokio runtime and runs the async main function inside it

async fn main () -> Result<(), Box<dyn Error>> {
    // sync fn (by defualt) init the server and start the app.
    // parentheses in the Result type indicate that the fn returns nothing on success, and Box<dyn Error> indicates that it can return any type of error that implements the Error trait.

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _) = broadcast::channel::<String>(100);

    // broadcast loop - main loop
    loop {
        // new connection
        let (socket, addr) = listener.accept().await?;

        //  log the new connection
        println!("New connection: {}", Local::now().format("%H:%M:%S"));
        println!("Address: {}", addr);

        // clonse sender and subscribe receiver for each new connection
        let tx = tx.clone();
        let rx = tx.subscribe();

        // spawn a new task to handle the client connection asynchronously
        tokio::spawn(async move {
            // moving any future msg or line etc
            handle_connection(socket, tx, rx).await.unwrap()
        });
    }
}
    // handle connection in a separate async task
    async fn handle_connection(
        mut socket: TcpStream,
        tx: broadcast::Sender<String>,
        mut rx: broadcast::Receiver<String>,
    ) -> Result<(), Box<dyn Error>> {

    let (reader, mut writer) = socket.split();
    // split the socket into a reader and writer, allowing for simultaneous reading and writing to the socket.
    // check difference with .into_split() and .split() in tokio
    let mut reader = BufReader::new(reader);
    let mut username = String::new();

    // read the username from the client
    reader.read_line(&mut username).await.unwrap();
    username = username.trim().to_string();

    // send a join message to all clients
    let join_message = ChatMessage {
        username: username.clone(),
        content: format!("{} has joined the chat", username),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::Join,
    };
    let join_json = serde_json::to_string(&join_message)?;
    tx.send(join_json).unwrap();

    // init a buffer for incoming messages 4rm the client
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break; // client disconnected
                }
                let msg = ChatMessage {
                    username: username.clone(),
                    content: line.trim().to_string(),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                    message_type: MessageType::UserMessage,
                };
                let msg_json = serde_json::to_string(&msg).unwrap();
                tx.send(msg_json).unwrap();
                line.clear();
            }
            // handle incoming broadcast and send 2 client
            result = rx.recv() => {
                let message = result?;
                writer.write_all(message.as_bytes()).await.unwrap(); // returns a slice of the msg
                writer.write_all(b"\n").await?; // write a newline after the message
            }
        }
    }

    // send a leave message to all clients
    let leave_message = ChatMessage {
        username: username.clone(),
        content: format!("{} has left the chat", username),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::Leave,
    };
    let leave_json = serde_json::to_string(&leave_message).unwrap();
    tx.send(leave_json).unwrap();

    // log the disconnection
    println!("[{}] {} is disconnected", Local::now().format("%H:%M:%S"), username);
        Ok(())
}