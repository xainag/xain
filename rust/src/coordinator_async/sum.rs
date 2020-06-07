use super::{CoordinatorState, RedisStore, State, StateError, StateMachine};
use crate::{
    coordinator::ProtocolEvent,
    coordinator_async::{
        error::Error,
        message::{MessageHandler, MessageSink, SumValidationData},
        update::Update,
    },
    crypto::generate_encrypt_key_pair,
    message::{MessageOwned, PayloadOwned},
    PetError,
};
use std::{default::Default, future::Future, pin::Pin, sync::Arc};
use tokio::{
    sync::{broadcast, mpsc},
    time::Duration,
};

pub struct Sum {
    sum_validation_data: Arc<SumValidationData>,
}

impl State<Sum> {
    pub fn new(
        coordinator_state: CoordinatorState,
        message_rx: mpsc::UnboundedReceiver<Vec<u8>>,
        redis: RedisStore,
        events_rx: mpsc::UnboundedSender<ProtocolEvent>,
    ) -> StateMachine {
        info!("state transition");
        let sum_validation_data = Arc::new(SumValidationData {
            seed: coordinator_state.seed.clone(),
            sum: coordinator_state.sum,
        });

        StateMachine::Sum(Self {
            _inner: Sum {
                sum_validation_data,
            },
            coordinator_state,
            message_rx,
            redis,
            events_rx,
        })
    }

    pub async fn next(mut self) -> StateMachine {
        info!("try to go to the next state");
        self.gen_round_keypair();
        self.set_coordinator_state().await;
        let _ = self.emit_round_parameters();

        match self.run_phase().await {
            Ok(_) => State::<Update>::new(
                self.coordinator_state,
                self.message_rx,
                self.redis,
                self.events_rx,
            ),
            Err(err) => State::<Error>::new(
                self.coordinator_state,
                self.message_rx,
                self.redis,
                self.events_rx,
                err,
            ),
        }
    }

    async fn run_phase(&mut self) -> Result<(), StateError> {
        let (sink_tx, sink) = MessageSink::new(
            self.coordinator_state.min_sum,
            Duration::from_secs(0),
            Duration::from_secs(10),
        );
        let (_cancel_complete_tx, mut cancel_complete_rx) = mpsc::channel::<()>(1);
        let (notify_cancel, _) = broadcast::channel::<()>(1);

        let phase_result = tokio::select! {
            message_source_result = async {
                loop {
                    let message = self.next_message().await?;
                    let message_handler = self.create_message_handler(
                        message, sink_tx.clone(),
                        _cancel_complete_tx.clone(),
                        notify_cancel.subscribe()
                    ).await?;
                    tokio::spawn(async move { message_handler.await });
                }
            } => {
                message_source_result
            }
            message_sink_result = sink.collect() => {
                message_sink_result
            }
        };

        // Drop the notify_cancel sender. By dropping the sender, all receivers will receive a
        // RecvError.
        drop(notify_cancel);

        // Wait until all MessageHandler tasks have been resolved/canceled.
        // (After all senders of this channel are dropped, which mean that all
        // MessageHandler have been dropped, the receiver of this channel will receive None).
        drop(_cancel_complete_tx);
        let _ = cancel_complete_rx.recv().await;

        phase_result?;
        self.emit_sum_dict().await
    }

    async fn create_message_handler(
        &mut self,
        message: MessageOwned,
        sink_tx: mpsc::UnboundedSender<Result<(), PetError>>,
        _cancel_complete_tx: mpsc::Sender<()>,
        notify_cancel: broadcast::Receiver<()>,
    ) -> Result<Pin<Box<dyn Future<Output = ()> + 'static + Send>>, PetError> {
        let participant_pk = message.header.participant_pk;
        let sum_message = match message.payload {
            PayloadOwned::Sum(msg) => msg,
            _ => return Err(PetError::InvalidMessage),
        };

        let message_handler =
            MessageHandler::new(sink_tx.clone(), _cancel_complete_tx.clone(), notify_cancel);

        let redis_connection = self.redis.clone().connection().await;

        Ok(Box::pin(message_handler.handle_sum_message(
            self._inner.sum_validation_data.clone(),
            participant_pk,
            sum_message,
            redis_connection,
        )))
    }

    /// Generate fresh round credentials.
    fn gen_round_keypair(&mut self) {
        let (pk, sk) = generate_encrypt_key_pair();
        self.coordinator_state.pk = pk;
        self.coordinator_state.sk = sk;
    }

    async fn emit_sum_dict(&self) -> Result<(), StateError> {
        // fetch sum dict
        let sum_dict = self
            .redis
            .clone()
            .connection()
            .await
            .get_sum_dict()
            .await
            .map_err(StateError::from)?;

        let _ = self.events_rx.send(ProtocolEvent::StartUpdate(sum_dict));
        Ok(())
    }

    fn emit_round_parameters(&self) {
        let _ = self
            .events_rx
            .send(ProtocolEvent::StartSum(self.round_parameters()));
    }
}
