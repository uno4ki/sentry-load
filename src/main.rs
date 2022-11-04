use std::env;
use std::thread;
use std::time::Duration;
use rand::Rng;
use std::process;
use std::io::{Error, ErrorKind};
use tracing;
use tracing_subscriber::prelude::*;


// Environment 
static DSN_ENV:     &str = "SL_DSN";
static RUNNERS_ENV: &str = "SL_RUNNERS";
static ERRORS_ENV:  &str = "SL_ERRORS";
static TXN_ENV:     &str = "SL_TRANSACTIONS";
static DELAY_ENV:   &str = "SL_DELAY";

// Defaults
const DEFAULT_RUNNERS:      u32 = 2;
const DEFAULT_ERRORS:       u32 = 2;
const DEFAULT_TRANSACTIONS: u32 = 5;
const DEFAULT_DELAY:        u32 = 2;

// Configuration
pub struct Config {
    pub dsn: String,
    pub runners: u32,
    pub errors: u32,
    pub transactions: u32,
    pub delay: u32,
}

// Load config
impl Config {
    pub fn configure() -> Result<Config, &'static str>{
        fn load_u32_env(name: &str, default: u32) -> u32 {
            match env::var_os(name) {
                Some(v) => v.into_string().unwrap().parse::<u32>().unwrap(),
                None => {
                    println!("${} not set, using default: {}", name, default);
                    default
                }
            }
        }

        Ok(Config {
            dsn: match env::var_os(DSN_ENV) {
                Some(v) => v.into_string().unwrap(),
                None => panic!("${} is not set", DSN_ENV)
            },
            runners: match env::var_os(RUNNERS_ENV) {
                Some(v) => v.into_string().unwrap().parse::<u32>().unwrap(),
                None => load_u32_env(RUNNERS_ENV, DEFAULT_RUNNERS)
            },
            errors: match env::var_os(ERRORS_ENV) {
                Some(v) => v.into_string().unwrap().parse::<u32>().unwrap(),
                None => load_u32_env(ERRORS_ENV, DEFAULT_ERRORS)
            },
            transactions: match env::var_os(TXN_ENV) {
                Some(v) => v.into_string().unwrap().parse::<u32>().unwrap(),
                None => load_u32_env(TXN_ENV, DEFAULT_TRANSACTIONS)
            },
            delay: match env::var_os(DELAY_ENV) {
                Some(v) => v.into_string().unwrap().parse::<u32>().unwrap(),
                None => load_u32_env(DELAY_ENV, DEFAULT_DELAY)
            }
        })
    }
}

fn error_runner(name: String, events: u32, delay: u32) {
    let mut rnd = rand::thread_rng();

    for seq in 0..events {
        let sleep: u64 = match delay {
            0 => 0,
            _ => rnd.gen_range(0..(delay * 1_000)).into()
        };

        sentry::capture_error(
            &Error::new(ErrorKind::Other, format!("Sentry error testing: name = {}, seq = seq_{}, sleep = {}",
                name, seq, sleep))
        );

        thread::sleep(Duration::from_millis(sleep));
    }
}

#[tracing::instrument]
fn tx3() {
    tracing::info!("tx level = 3");
    thread::sleep(Duration::from_micros(10));
}

#[tracing::instrument]
fn tx2(depth: u8) {
    tracing::info!("tx level = 2");
    if depth > 1 {
        tx3()
    }
    thread::sleep(Duration::from_micros(10));
}

#[tracing::instrument]
fn tx1(depth: u8) {
    tracing::info!("tx level = 1");
    if depth > 0 {
        tx2(depth)
    }

    thread::sleep(Duration::from_micros(10));
}

#[tracing::instrument]
fn tx0(name: String, depth: u8) {
    tracing::info!(name);
    tx1(depth)
}
fn txn_runner(name: String, events: u32, delay: u32) {
    let mut rnd = rand::thread_rng();

    for seq in 0..events {
        let sleep: u64 = match delay {
            0 => 0,
            _ => rnd.gen_range(0..(delay * 1_000_00)).into()
        };

        tx0(
            format!("tx: name = {}, seq = seq_{}, sleep = {}, level = 0", name, seq, sleep),
            rnd.gen_range(0..2).into()
        );
        
        thread::sleep(Duration::from_micros(sleep));
    }
}

fn main() {
    let cfg: Config = Config::configure().unwrap();

    println!("DSN: {}", cfg.dsn);
    println!("Runners: {}", cfg.runners);
    println!("Errors: {}", cfg.errors);
    println!("Transactions: {}", cfg.transactions);
    println!("Delay max: {}", cfg.delay);

    // Initialize sentry
    let _sentry = sentry::init((cfg.dsn, sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: 1.0,
        ..Default::default()
    }));

    // Initialize tracer
    tracing_subscriber::registry()
        .with(sentry_tracing::layer())
        .init();

    // Initialize loops
    let mut hndls = vec![];

    for seq in 0..cfg.runners {
        hndls.push(thread::spawn(move || error_runner(format!("err_{}_{}", seq, process::id()), cfg.errors, cfg.delay)));
        hndls.push(thread::spawn(move || txn_runner(format!("txn_{}_{}", seq, process::id()), cfg.transactions, cfg.delay)));
    }

    // Wait for the threads
    for h in hndls {
        h.join().unwrap();
    }
}
