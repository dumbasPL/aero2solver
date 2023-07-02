use aero2solver::{
    constants::{BASE_URL, USER_AGENT},
    portal::PortalClient,
    solver::Aero2Solver,
};
use anyhow::{anyhow, Result};
use argh::FromArgs;
use std::{thread, time::Duration};

#[derive(FromArgs)]
/// Solve Aero2 captchas automatically.
struct Args {
    /// model configuration file.
    /// default: "./model/captcha.cfg"
    #[argh(option, short = 'c', default = "String::from(\"./model/captcha.cfg\")")]
    model_cfg: String,

    /// model weights file.
    /// default: "./model/captcha.weights"
    #[argh(
        option,
        short = 'w',
        default = "String::from(\"./model/captcha.weights\")"
    )]
    weights: String,

    /// model labels file.
    /// default: "./model/captcha.names"
    #[argh(
        option,
        short = 'l',
        default = "String::from(\"./model/captcha.names\")"
    )]
    labels: String,

    /// minimum confidence threshold for captcha detection (0.0 - 1.0).
    /// default: 0.8
    #[argh(option, short = 't', default = "0.8")]
    threshold: f32,

    /// time to wait after aero2 returns an error (in seconds).
    /// default: 5.0
    #[argh(option, default = "5.0")]
    error_delay: f32,

    /// time to wait between checking for new captchas (in seconds).
    /// default: 10.0
    #[argh(option, default = "10.0")]
    check_delay: f32,

    /// time to wait after successfully solving a captcha (in seconds).
    /// default: 3300 (55 minutes)
    #[argh(option, default = "55.0 * 60.0 // 55 minutes")]
    solved_delay: f32,
}

fn main() -> Result<()> {
    let Args {
        model_cfg,
        weights,
        labels,
        error_delay,
        check_delay,
        solved_delay,
        threshold,
    } = argh::from_env();

    if threshold < 0.0 || threshold > 1.0 {
        return Err(anyhow!("Threshold must be between 0.0 and 1.0"));
    }

    let mut solver = Aero2Solver::new(&labels, &model_cfg, &weights)?;

    let error_sleep_time = Duration::from_secs_f32(error_delay);
    let check_sleep_time = Duration::from_secs_f32(check_delay);
    let solved_sleep_time = Duration::from_secs_f32(solved_delay);

    loop {
        let was_solved = run(&mut solver, threshold, error_sleep_time).unwrap_or_else(|x| {
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

fn run(solver: &mut Aero2Solver, min_threshold: f32, fail_sleep_time: Duration) -> Result<bool> {
    let client = PortalClient::new(BASE_URL, USER_AGENT)?;

    let mut was_required = false;

    let mut state = client.get_state()?;
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

        let mut tries = 0;
        let solution: String = loop {
            tries += 1;
            if tries > 20 {
                break Err(anyhow!("Too many tries"));
            }
            println!("Trying to solve captcha (try {})", tries);
            let captcha = client.get_captcha(&state.session_id)?;
            match solver.solve(&captcha, min_threshold, 8) {
                Ok(solution) => {
                    println!("Captcha solved as {} after {}", solution, tries);
                    break Ok(solution);
                }
                Err(e) => println!("Error while solving captcha: {}", e),
            }
        }?;

        state = client.submit_captcha(&state.session_id, &solution)?;
    }
}
