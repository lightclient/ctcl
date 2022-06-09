use beacon_api_client::{Client, PubkeyOrIndex, StateId, ValidatorDescriptor, ValidatorStatus};
use clap::{self, Parser};
use url::Url;

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,

    #[clap(
        short,
        long,
        default_value = "http://127.0.0.1:5052",
        parse(try_from_str)
    )]
    beacon: Url,
}

#[derive(Debug, clap::Subcommand)]
enum Action {
    #[clap(subcommand)]
    Validator(ValidatorActions),
}

#[derive(Debug, clap::Subcommand)]
enum ValidatorActions {
    Activation { index: u64 },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.action {
        Action::Validator(action) => match action {
            ValidatorActions::Activation { index } => {
                let client = Client::new(args.beacon);
                let validator = client
                    .get_validator(StateId::Head, PubkeyOrIndex::Index(index as usize))
                    .await
                    .unwrap();
                println!("{:?}", validator.validator);
                let pending = client
                    .get_validators(
                        StateId::Head,
                        &[ValidatorDescriptor::Status(ValidatorStatus::Pending)],
                    )
                    .await
                    .unwrap();
                println!("{:?}", pending.len())
            }
        },
    }
}
