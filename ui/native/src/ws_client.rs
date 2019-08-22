use futures::future::Future;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};
use data_model::{CmdMsg, UpdateMsg};

pub fn connect(address: String, input: mpsc::Receiver<CmdMsg>, output: crossbeam_channel::Sender<UpdateMsg>) {
    println!("Connecting on {:?}", address);
	std::thread::spawn(move || {
        let mut runtime = tokio::runtime::current_thread::Builder::new()
            .build()
            .unwrap();

        let runner = ClientBuilder::new(&address)
            .unwrap()
            .add_protocol("rust-websocket")
            .async_connect_insecure()
            .and_then(|(duplex, _)| {
                let (sink, stream) = duplex.split();
                stream
                    .filter_map(|message| {
                        match message {
                            OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                            OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
                            OwnedMessage::Text(msg) => {
                                println!("Received Message: {:?}", msg);
                                let update: UpdateMsg = serde_json::from_str(&msg).unwrap();
                                output.send(update).unwrap();
                                None
                            }
                            _ => None,
                        }
                    })
                    .select(input.map(|msg| {
                        println!("Sending message {:?}", msg);
                        OwnedMessage::Text(serde_json::to_string(&msg).unwrap())
                    }).map_err(|_| WebSocketError::NoDataAvailable))
                    .forward(sink)
            });
        runtime.block_on(runner).unwrap();
	});
}
