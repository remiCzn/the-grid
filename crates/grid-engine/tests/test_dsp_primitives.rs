use grid_engine::dsp::{FilterMode, Oscillator, StateVariableFilter, VcaGain, Waveform};

#[test]
fn test_oscillator_waveforms_and_frequency() {
    let mut osc = Oscillator::new(48000.0);
    osc.set_frequency(440.0);

    for wf in [
        Waveform::Sine,
        Waveform::Sawtooth,
        Waveform::Square,
        Waveform::Triangle,
    ] {
        osc.waveform = wf;
        let mut buffer = [0.0f32; 128];
        osc.process_block(&mut buffer, None);

        let max_val = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(
            max_val > 0.5,
            "L'oscillateur ({:?}) doit générer un signal non nul",
            wf
        );
        assert!(
            max_val <= 1.2,
            "L'oscillateur ({:?}) ne doit pas dépasser les bornes",
            wf
        );
    }
}

#[test]
fn test_oscillator_pitch_cv_modulation() {
    let mut osc = Oscillator::new(48000.0);
    osc.set_frequency(440.0);

    let cv = [1.0f32; 64];
    let mut buffer = [0.0f32; 64];
    osc.process_block(&mut buffer, Some(&cv));

    let max_val = buffer.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(max_val > 0.5);
}

#[test]
fn test_vca_gain_and_cv() {
    let mut vca = VcaGain::new();
    vca.set_gain(0.5);

    let input = [1.0f32; 64];
    let mut output = [0.0f32; 64];

    vca.process_block(&input, &mut output, None);

    let last_sample = output[63];
    assert!((last_sample - 0.5).abs() < 0.1);

    let cv = [-1.0f32; 64];
    vca.process_block(&input, &mut output, Some(&cv));
    let zero_sample = output[63];
    assert_eq!(zero_sample, 0.0);
}

#[test]
fn test_svf_filter_stability_and_all_modes() {
    let mut svf = StateVariableFilter::new(48000.0);
    svf.set_cutoff(1000.0);
    svf.set_resonance(2.0);

    for mode in [
        FilterMode::LowPass,
        FilterMode::HighPass,
        FilterMode::BandPass,
        FilterMode::Notch,
    ] {
        svf.mode = mode;
        svf.reset();

        let input = [0.5f32; 128];
        let mut output = [0.0f32; 128];
        svf.process_block(&input, &mut output);

        for &s in &output {
            assert!(!s.is_nan(), "Filtre ne doit pas produire de NaN");
            assert!(!s.is_infinite(), "Filtre ne doit pas produire d'Inf");
        }
    }
}
