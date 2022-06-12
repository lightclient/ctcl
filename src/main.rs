use beacon_api_client::{Client, PubkeyOrIndex, StateId, ValidatorStatus};
use chrono::prelude::*;
use clap::{self, Parser};
use ethereum_consensus::{configs, phase0::mainnet::SLOTS_PER_EPOCH};
use futures::future::try_join;
use std::process::exit;
use url::Url;

#[derive(Debug, clap::ArgEnum, Clone)]
/// Network configuration.
enum Config {
    Mainnet,
    Minimal,
}

#[derive(Debug, clap::Parser)]
#[clap(rename_all = "kebab")]
struct Args {
    #[clap(subcommand)]
    action: Action,

    #[clap(
        short,
        long,
        default_value = "http://127.0.0.1:5052",
        parse(try_from_str)
    )]
    beacon_api: Url,

    #[clap(short, long, default_value = "mainnet", arg_enum, parse(try_from_str))]
    config: Config,
}

#[derive(Debug, clap::Subcommand)]
/// Subcommands for ctcl.
enum Action {
    #[clap(subcommand)]
    Validator(ValidatorActions),
}

#[derive(Debug, clap::Subcommand)]
/// Commands to query validator information.
enum ValidatorActions {
    /// Check when a pending validator will become active.
    Activation {
        /// Validator index.
        index: u64,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = match args.config {
        Config::Mainnet => configs::mainnet::config(),
        Config::Minimal => panic!("minimal config not supported"),
    };

    match args.action {
        Action::Validator(action) => match action {
            ValidatorActions::Activation { index } => {
                let client = Client::new(args.beacon_api);
                let validator = client
                    .get_validator(StateId::Head, PubkeyOrIndex::Index(index as usize))
                    .await
                    .unwrap();

                match validator.status {
                    ValidatorStatus::Active
                    | ValidatorStatus::ActiveOngoing
                    | ValidatorStatus::ActiveExiting
                    | ValidatorStatus::ActiveSlashed => {
                        println!("validator {}", validator.status);
                        exit(0);
                    }
                    _ => (),
                }

                let active_fut =
                    client.get_validators(StateId::Head, &[], &[ValidatorStatus::ActiveOngoing]);
                let pending_fut =
                    client.get_validators(StateId::Head, &[], &[ValidatorStatus::PendingQueued]);

                let (active, mut pending) = match try_join(active_fut, pending_fut).await {
                    Ok((a, p)) => (a, p),
                    Err(err) => {
                        eprintln!("{}", err);
                        exit(1);
                    }
                };

                pending.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());
                let diff = index - pending[0].index as u64;

                println!(
                    "waiting on {} validators in a queue of {} validators...",
                    diff,
                    pending.len()
                );

                let churn_limit = u64::max(
                    config.min_per_epoch_churn_limit,
                    active.len() as u64 / config.churn_limit_quotient,
                );
                let epochs = diff / churn_limit;
                let duration = chrono::Duration::seconds(
                    (epochs * (config.seconds_per_slot * SLOTS_PER_EPOCH)) as i64,
                );
                let eta = Local::now() + duration;

                println!(
                    "estimate activation time: {} ({}d {}h {}m)",
                    eta.format("%b %d %Y @ %r"),
                    duration.num_days(),
                    duration.num_hours() % 24,
                    duration.num_minutes() % 60,
                );
            }
        },
    }
}
