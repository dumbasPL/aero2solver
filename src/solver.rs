use anyhow::{anyhow, Result};
use darknet::{Image, Network};
use std::{fs, path::Path};

pub struct Aero2Solver {
    object_labels: Vec<String>,
    net: Network,
    min_threshold: f32,
    expected_length: usize,
}

impl Aero2Solver {
    pub fn new<P: AsRef<Path>>(
        names_file: P,
        config_file: P,
        model_file: P,
        min_threshold: f32,
        expected_length: usize,
    ) -> Result<Self> {
        let object_labels = fs::read_to_string(names_file)?
            .lines()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let net = Network::load(config_file, Some(model_file), false)?;

        Ok(Self {
            object_labels,
            net,
            min_threshold,
            expected_length,
        })
    }

    pub fn solve(&mut self, captcha: &[u8]) -> Result<String> {
        let captcha_image: Image = image::load_from_memory(&captcha)?.into();

        let detections = self.net.predict(captcha_image, 0.25, 0.5, 0.45, true);
        let mut detections: Vec<_> = detections
            .iter()
            .flat_map(|det| {
                det.best_class(Some(self.min_threshold))
                    .map(|(class_index, prob)| (det, prob, &self.object_labels[class_index]))
            })
            .collect();

        if detections.len() != self.expected_length {
            return Err(anyhow!(
                "Invalid captcha length. Expected: {}, Got: {}",
                self.expected_length,
                detections.len()
            ));
        }

        // sport by x
        detections.sort_by(|a, b| {
            let a = a.0.bbox().x;
            let b = b.0.bbox().x;
            a.partial_cmp(&b).unwrap()
        });

        Ok(detections
            .into_iter()
            .map(|(_, _, class)| class)
            .cloned()
            .collect())
    }
}
