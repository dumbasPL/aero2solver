use aero2solver::solver::Aero2Solver;
use std::fs;

#[test]
fn test_solve() {
    let mut solver = Aero2Solver::new(
        "./model/captcha.names",
        "./model/captcha.cfg",
        "./model/captcha.weights",
    )
    .unwrap();

    // solve all captchas that match the pattern captcha_*.jpg
    let files = fs::read_dir("./tests/data")
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|x| {
            x.to_str().unwrap().contains("captcha_") && x.to_str().unwrap().ends_with(".jpg")
        })
        .collect::<Vec<_>>();

    for ref file in files {
        // extract text between captcha_ and .jpg
        let correct_solution = file
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split('_')
            .last()
            .unwrap();

        let captcha = fs::read(file).unwrap();
        let solution = solver.solve(&captcha, 0.9, 8).unwrap();
        assert_eq!(solution, correct_solution);
        println!("solved test captcha: {}", solution);
    }
}
