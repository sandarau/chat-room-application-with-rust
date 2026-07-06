#[macro_use] extern crate rocket;

use rocket::{State, Shutdown};
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::select;
use rocket::fs::{relative, FileServer};
use rocket::form::Form;
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![post, events])
        .mount("/", FileServer::from(relative!("front-end")))// a handler that'll serve static files
}
// manage method allows adding state to the rocket server instance, which all handlers have access to. In this case, we are adding a channel that can be used to send messages between different parts of the application. The channel has a buffer size of 1024, which means it can hold up to 1024 messages before blocking.
// rocket uses Tokyo as async runtime. the return val of calling the channel function is a tuple containing a Sender and a Receiver. to store the sender: use .0 to get the first element of the tuple.
// rocket provides supposrt for returning html, css, javascrpt

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(1..=20))]
    pub room: String,
    #[field(validate = len(1..=20))]//19 char long
    pub username: String,
    //#[field(validate = len(1..=100))]
    pub message: String,
}

// post msgs
// Rocket route handler for POST requests to the /message endpoint. It takes a Form<Message> as input and a reference to the Sender<Message> state. The form data is converted into a Message struct and sent through the channel using the send method of the Sender. The result of the send operation is ignored, as we are not handling any errors in this example.
#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let _res = queue.send(form.into_inner());// send msg to all receivers
}

// receive (get) msgs; handle "get" requests to the events path
// the return type is an infinite stream of server-sent events "EventStream" that can be consumed by clients. The stream is created using the broadcast channel's subscribe method, which returns a Receiver<Message> that can be used to receive messages sent through the channel. The stream is then mapped to convert each Message into a Server-Sent Event (SSE) using the Event::data method, which takes a string as input. The resulting stream is returned as an EventStream, which can be consumed by clients using JavaScript's EventSource API.
// EventStream allow clients to open a long-lived connection with the server, and then the server can send data to the clients whenever it wants. 
// his is similar to WebSockets, except it only works in one direction. The server can send data to clients, but the clients can't send data back to the server.
#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select!{//waits on many concurrent branches and returns soon as one of em is completed:
                msg = rx.recv() => match msg {// a branch that waits for a message to be received from the channel. The recv method of the Receiver<Message> is called, which blocks until a message is available or the channel is closed.
                    // result of the recv method is an enum that's matched across 3 possible types
                    Ok(msg) => msg,// msg received successfully
                    Err(RecvError::Closed) => break,// if the channel was closed
                    Err(RecvError::Lagged(_)) => continue,// receiver has lagged, the loop continues to wait for the next message.
                },
                _ = &mut end => break,// 2nd branch that waits for Shutdown signal, provided by rocket, to gracefully shut down the server and EventStream ends
            };// break out of infinite loop

            yield Event::json(&msg)
                
        }
    }
}
