use tracing_subscriber::*;
use xain_fl::{
    client::{Client, ClientError},
    mask::{FromPrimitives, Model},
    participant::Task,
};

/// Test-drive script of a (local, but networked) single-round federated
/// learning session, intended for use as a mini integration test. It assumes
/// that a [`Service`] is already running and listening to
/// http://127.0.0.1:3030.
///
/// 10 [`Client`]s are spawned on the tokio event loop. This serves as a simple
/// example of getting started with the project, and may later be the basis for
/// more automated tests.
///
/// important NOTE since we only test a few clients and by default, the
/// selection ratios in the Coordinator are relatively small, it is very
/// possible no (or too few) participants will be selected here! It's currently
/// not possible to configure or force the selection, hence as a TEMP
/// workaround, these should be adjusted in coordinator.rs before running this
/// test e.g. 0.2_f64 for sum and 0.4_f64 for update.
#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let _fmt_subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    // dummy local model for clients
    let model = Model::from_primitives(vec![0_f32, 1_f32, 0_f32, 1_f32].into_iter()).unwrap();

    let mut tasks = vec![];
    for id in 0..10 {
        let mut client = Client::new_with_addr(1, id, "http://127.0.0.1:3030")?;
        client.local_model = Some(model.clone());
        let join_hdl = tokio::spawn(async move { client.during_round().await });
        tasks.push(join_hdl);
    }
    println!("spawned 10 clients");

    let mut summers = 0;
    let mut updaters = 0;
    let mut unselecteds = 0;
    for task in tasks {
        match task.await.or(Err(ClientError::GeneralErr))?? {
            Task::Update => updaters += 1,
            Task::Sum => summers += 1,
            Task::None => unselecteds += 1,
        }
    }

    println!(
        "{} sum, {} update, {} unselected clients completed a round",
        summers, updaters, unselecteds
    );

    Ok(())
}
