use aero2solver::{
    constants::{BASE_URL, USER_AGENT},
    portal::PortalClient,
    solver::Aero2Solver,
};
use anyhow::{anyhow, Result};
use clap::Parser;
use std::{path::PathBuf, process, thread, time::Duration};

fn get_model_path(filename: &str) -> String {
    let model_path = option_env!("MODEL_PATH").unwrap_or("./model");
    let mut path = PathBuf::from(model_path);
    path.push(filename);
    path.to_str().unwrap().to_string()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// model configuration file.
    #[arg(short = 'c', long, env = "AERO2_MODEL_CFG", default_value_t = get_model_path("captcha.cfg"))]
    model_cfg: String,

    /// model weights file.
    #[arg(short = 'w', long, env = "AERO2_WEIGHTS", default_value_t = get_model_path("captcha.weights"))]
    weights: String,

    /// model labels file.
    #[arg(short = 'l', long, env = "AERO2_LABELS", default_value_t = get_model_path("captcha.names"))]
    labels: String,

    /// minimum confidence threshold for captcha detection (0.0 - 1.0).
    #[arg(short = 't', long, env = "AERO2_THRESHOLD", default_value_t = 0.8)]
    threshold: f32,

    /// time to wait after aero2 returns an error (in seconds).
    #[arg(long, env = "AERO2_ERROR_DELAY", default_value_t = 10.0)]
    error_delay: f32,

    /// time to wait between checking for new captchas (in seconds).
    #[arg(long, env = "AERO2_CHECK_DELAY", default_value_t = 10.0)]
    check_delay: f32,

    /// time to wait after successfully solving a captcha (in seconds).
    #[arg(long, env = "AERO2_SOLVED_DELAY", default_value_t = 60.0)]
    solved_delay: f32,

    /// test captcha solving without actually submitting it
    #[arg(
        short = 'd',
        long,
        value_name = "count",
        num_args = 0..=1,
        default_missing_value = "1"
    )]
    dry_run: Option<u32>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let Args {
        model_cfg,
        weights,
        labels,
        error_delay,
        check_delay,
        solved_delay,
        threshold,
        dry_run,
    } = Args::parse();

    if threshold < 0.0 || threshold > 1.0 {
        return Err(anyhow!("Threshold must be between 0.0 and 1.0"));
    }

    let mut solver = Aero2Solver::new(&labels, &model_cfg, &weights, threshold, 8)?;

    if let Some(count) = dry_run {
        let mut errored = false;
        for i in 0..count {
            println!("Running test {}/{}", i + 1, count);
            if let Err(e) = run_test(&mut solver).await {
                println!("Error: {}", e);
                errored = true;
            }
        }
        if errored {
            process::exit(1);
        }
        return Ok(());
    }

    let error_sleep_time = Duration::from_secs_f32(error_delay);
    let check_sleep_time = Duration::from_secs_f32(check_delay);
    let solved_sleep_time = Duration::from_secs_f32(solved_delay);

    loop {
        let was_solved = run(&mut solver, error_sleep_time)
            .await
            .unwrap_or_else(|x| {
                println!("Error: {}", x);
                false
            });

        let sleep_time = match was_solved {
            true => {
                println!("Sleeping for {} seconds", solved_sleep_time.as_secs_f32());
                solved_sleep_time
            }
            false => check_sleep_time,
        };

        thread::sleep(sleep_time);
    }
}

async fn run_test(solver: &mut Aero2Solver) -> Result<()> {
    let client = PortalClient::new(BASE_URL, USER_AGENT)?;

    solve_captcha(solver, &client, None, 20).await?;

    Ok(())
}

async fn run(solver: &mut Aero2Solver, fail_sleep_time: Duration) -> Result<bool> {
    let client = PortalClient::new(BASE_URL, USER_AGENT)?;

    let mut was_required = false;

    let mut state = client.get_state().await?;
    loop {
        if !state.captcha_present {
            let status = match was_required {
                true => "Captcha solved",
                false => "Captcha not required",
            };
            match state.message {
                Some(ref message) => println!("{} with message: {}", status, message),
                None => println!("{}", status),
            }
            break Ok(was_required);
        }

        was_required = true;

        match state.message {
            Some(ref message) => {
                println!(
                    "Captcha required with message: {}. Waiting for {} seconds before retrying",
                    message,
                    fail_sleep_time.as_secs_f32()
                );
                thread::sleep(fail_sleep_time);
            }
            None => println!("Captcha required"),
        }

        let solution = solve_captcha(solver, &client, Some(&state.session_id), 20).await?;

        state = client.submit_captcha(&state.session_id, &solution).await?;
    }
}

async fn solve_captcha(
    solver: &mut Aero2Solver,
    client: &PortalClient,
    session_id: Option<&str>,
    max_tries: u32,
) -> Result<String> {
    let mut tries = 0;
    let solution: String = loop {
        tries += 1;
        if tries > max_tries {
            break Err(anyhow!("Too many tries"));
        }
        println!("Trying to solve captcha (try {})", tries);
        let captcha = client.get_captcha(session_id).await?;
        match solver.solve(&captcha) {
            Ok(solution) => {
                match tries {
                    1 => println!("Captcha solved as {} after {} try", solution, tries),
                    _ => println!("Captcha solved as {} after {} tries", solution, tries),
                }
                break Ok(solution);
            }
            Err(e) => println!("Error while solving captcha: {}", e),
        }
    }?;

    Ok(solution)
}
