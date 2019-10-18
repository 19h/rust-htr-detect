use biquad;
use biquad::{Biquad, ToHertz, Hertz};

use std::default::Default;

mod stats;
use stats::Stats;

use smoothed_z_score::{Peak, PeaksDetector, PeaksFilter};

use sample;
use sample::Signal;

struct HT<'a> {
    // Fractioning

    // Fs
    frame_size: f64,

    // frac
    interval: f64,

    // Band-pass filter parameters

    // bpL
    band_pass_low: f64,

    // bpL
    band_pass_high: f64,

    // HTR detection parameters

    // nSD
    ht_std_dev: f64,

    // Ttv
    ht_top_threshold: f64,

    // mpd
    ht_event_min_distance: f64,

    // Mpw
    ht_event_width: f64,

    data: &'a [f64],
}

impl<'a> Default for HT<'a> {
    fn default() -> HT<'a> {
        HT {
            frame_size: 1.0,
            interval: 15.0,

            band_pass_low: 70.0,
            band_pass_high: 110.0,

            ht_std_dev: 15.0,
            ht_top_threshold: 0.075,
            ht_event_min_distance: 200.0,
            ht_event_width: 90.0,

            data: &[],
        }
    }
}

impl<'a> HT<'a> {
    fn new(data: &'a [f64]) -> HT<'a>{
        HT {
            data,

            frame_size: data.len() as f64,

            ..Default::default()
        }
    }

    fn process(&self) {
        // apply butterworth band pass filter to raw data
        let mut data = {
            let btrwrth_coeffs = biquad::Coefficients::<f64>::from_params(
                biquad::Type::LowPass,
                Hertz::<f64>::from_hz(self.frame_size).unwrap(),
                Hertz::<f64>::from_hz(self.interval).unwrap(),
                biquad::Q_BUTTERWORTH_F64,
            ).unwrap();

            let mut bq2 = biquad::DirectForm2Transposed::<f64>::new(
                btrwrth_coeffs,
            );

            self.data
                .iter()
                .map(|item|
                    bq2.run(*item)
                )
                .collect::<Vec<f64>>()
        };

        // baseline correction & transformation to absolute values
        let mean = data.mean();

        let mut absh: Vec<f64> = vec!(0.0; data.len());

        data
            .iter_mut()
            .enumerate()
            .for_each(|(i, item)| {
                *item = *item - mean;

                println!("{:?} {:?}", item, mean);

                absh[i] = (*item).abs();
            });

        // setting thresholds
        let sd = data.std_dev();

        let treshold = {
            if sd * self.ht_std_dev > self.ht_top_threshold {
                self.ht_top_threshold
            } else {
                sd * self.ht_std_dev
            }
        };

        // find local maxima (prominence, time indices)
        let locmax = absh
            .into_iter()
            .enumerate()
            .peaks(PeaksDetector::new(
                0,
                self.ht_std_dev,
                self.ht_top_threshold
            ), |e| e.1)
            .collect::<Vec<_>>();

        println!("-2 locmax {:?}", locmax);

        // detect local maximal for HTR events

        // total count

        // fractioned count
    }
}

fn main() {
    let data = sample::signal::rate(15.0)
        .const_hz(1.0)
        .sine()
        .into_interleaved_samples()
        .into_iter()
        .take(1000)
        .collect::<Vec<f64>>();

    let ht = HT::new(
        &data,
    );

    ht.process();
}
