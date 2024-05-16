use std::{
    fs::{create_dir_all, File},
    io::Write,
    time::Duration,
};

use aero2solver::{
    constants::{BASE_URL, USER_AGENT},
    portal::PortalClient,
    solver::Aero2Solver,
};
use anyhow::{anyhow, Result};

async fn run(solver: &mut Aero2Solver) -> Result<()> {
    let client = PortalClient::new(BASE_URL, USER_AGENT, Duration::from_secs(60))?;

    let state = client.get_state().await?;

    let mut tries = 0;
    let (solution, captcha) = loop {
        tries += 1;
        println!("Trying to solve captcha (try {})", tries);
        let captcha = client.get_captcha(Some(&state.session_id)).await?;
        match solver.solve(&captcha) {
            Ok(solution) => {
                println!("Captcha solved as {} after {}", solution, tries);
                break (solution, captcha);
            }
            Err(e) => println!("Error while solving captcha: {}", e),
        }
    };

    let state = client.submit_captcha(&state.session_id, &solution).await?;
    match state.message {
        Some(ref message) if message.eq("Rozłącz i ponownie połącz się z Internetem.") => {
            println!("Captcha solved, code: {}", solution);
        }
        _ => Err(anyhow!(
            "Captcha solved with message: {}",
            state.message.unwrap_or_default()
        ))?,
    }

    let mut file = File::create(format!("./tests/data/captcha_{}.jpg", solution))?;
    file.write_all(&captcha)?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut solver = Aero2Solver::new(
        "./model/captcha.names",
        "./model/captcha.cfg",
        "./model/captcha.weights",
        0.9,
        8,
    )?;

    create_dir_all("./tests/data")?;

    // loop until we have 10 successful solves
    let mut success_count = 0;
    loop {
        if success_count >= 10 {
            break;
        }

        match run(&mut solver).await {
            Ok(_) => success_count += 1,
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
