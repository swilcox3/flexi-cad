use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};
use data_model::{CmdMsg, UpdateMsg};

pub fn connect(address: String, input: mpsc::Receiver<CmdMsg>, output: crossbeam_channel::Sender<UpdateMsg>) {
	std::thread::spawn(|| {
        let mut runtime = tokio::runtime::current_thread::Builder::new()
            .build()
            .unwrap();

        let runner = ClientBuilder::new(address)
            .unwrap()
            .add_protocol("rust-websocket")
            .async_connect_insecure()
            .and_then(|(duplex, _)| {
                let (sink, stream) = duplex.split();
                stream
                    .filter_map(|message| {
                        println!("Received Message: {:?}", message);
                        match message {
                            OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                            OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
                            OwnedMessage::Text(msg) => {
                                let update: UpdateMsg = serde_json::from_str(&msg).unwrap();
                                output.send(update).unwrap();
                                None
                            }
                            _ => None,
                        }
                    })
                    .select(input.map(|msg| {
                        OwnedMessage::Text(serde_json::to_string(&msg))
                    }).map_err(|_| WebSocketError::NoDataAvailable))
                    .forward(sink)
            });
        runtime.block_on(runner).unwrap();
	});
}
