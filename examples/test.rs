use aero2solver::{
    constants::{BASE_URL, USER_AGENT},
    portal::PortalClient,
    solver::Aero2Solver,
};
use anyhow::Result;

async fn run(solver: &mut Aero2Solver) -> Result<()> {
    let client = PortalClient::new(BASE_URL, USER_AGENT)?;

    let state = client.get_state().await?;

    let mut tries = 0;
    let solution: String = loop {
        tries += 1;
        println!("Trying to solve captcha (try {})", tries);
        let captcha = client.get_captcha(&state.session_id).await?;
        match solver.solve(&captcha, 0.8, 8) {
            Ok(solution) => {
                println!("Captcha solved as {} after {}", solution, tries);
                break solution;
            }
            Err(e) => println!("Error while solving captcha: {}", e),
        }
    };

    let state = client.submit_captcha(&state.session_id, &solution).await?;
    println!(
        "Captcha solved with message: {}",
        state.message.unwrap_or_default()
    );

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut solver = Aero2Solver::new(
        "./model/captcha.names",
        "./model/captcha.cfg",
        "./model/captcha.weights",
    )?;

    loop {
        run(&mut solver).await?;
    }
}
