use grid_core::model::ModuleKind;
use grid_engine::dsp::{ConstantNode, DspNode};

#[test]
fn test_constant_node_generation_and_clamping() {
    let mut node = ConstantNode::new(0.5);
    assert_eq!(node.value, 0.5);

    node.set_value(2.0); // Clamped to 1.0
    assert_eq!(node.value, 1.0);

    node.set_value(-2.0); // Clamped to -1.0
    assert_eq!(node.value, -1.0);
}

#[test]
fn test_dsp_node_factory_and_parameter_encapsulation() {
    let sample_rate = 48000.0;

    // 1. Oscillator node
    let mut osc_node = DspNode::from_kind(ModuleKind::Oscillator, sample_rate, &[220.0]);
    if let DspNode::Oscillator(ref osc) = osc_node {
        assert_eq!(osc.frequency, 220.0);
    } else {
        panic!("Expected Oscillator node");
    }
    osc_node.set_parameter(0, 880.0);
    if let DspNode::Oscillator(ref osc) = osc_node {
        assert_eq!(osc.frequency, 880.0);
    }

    // 2. Gain node
    let mut gain_node = DspNode::from_kind(ModuleKind::Gain, sample_rate, &[0.75]);
    if let DspNode::Gain(ref vca) = gain_node {
        assert_eq!(vca.target_gain, 0.75);
    } else {
        panic!("Expected Gain node");
    }
    gain_node.set_parameter(0, 1.5);
    if let DspNode::Gain(ref vca) = gain_node {
        assert_eq!(vca.target_gain, 1.5);
    }

    // 3. Filter node
    let mut filter_node = DspNode::from_kind(ModuleKind::Filter, sample_rate, &[500.0, 2.0]);
    if let DspNode::Filter(ref svf) = filter_node {
        assert_eq!(svf.cutoff, 500.0);
        assert_eq!(svf.resonance, 2.0);
    } else {
        panic!("Expected Filter node");
    }
    filter_node.set_parameter(0, 1200.0);
    filter_node.set_parameter(1, 4.0);
    if let DspNode::Filter(ref svf) = filter_node {
        assert_eq!(svf.cutoff, 1200.0);
        assert_eq!(svf.resonance, 4.0);
    }

    // 4. Envelope node
    let mut env_node =
        DspNode::from_kind(ModuleKind::Envelope, sample_rate, &[15.0, 50.0, 0.6, 300.0]);
    if let DspNode::Envelope(ref env) = env_node {
        assert_eq!(env.attack_time_ms, 15.0);
        assert_eq!(env.decay_time_ms, 50.0);
        assert_eq!(env.sustain_level, 0.6);
        assert_eq!(env.release_time_ms, 300.0);
    } else {
        panic!("Expected Envelope node");
    }
    env_node.set_parameter(0, 20.0);
    env_node.set_parameter(1, 80.0);
    env_node.set_parameter(2, 0.5);
    env_node.set_parameter(3, 400.0);
    if let DspNode::Envelope(ref env) = env_node {
        assert_eq!(env.attack_time_ms, 20.0);
        assert_eq!(env.decay_time_ms, 80.0);
        assert_eq!(env.sustain_level, 0.5);
        assert_eq!(env.release_time_ms, 400.0);
    }

    // 5. Constant node
    let mut const_node = DspNode::from_kind(ModuleKind::Constant, sample_rate, &[0.42]);
    if let DspNode::Constant(ref c) = const_node {
        assert_eq!(c.value, 0.42);
    } else {
        panic!("Expected Constant node");
    }
    const_node.set_parameter(0, -0.8);
    if let DspNode::Constant(ref c) = const_node {
        assert_eq!(c.value, -0.8);
    }
}

#[test]
fn test_dsp_node_uniform_process_block() {
    let sample_rate = 48000.0;

    // 1. Constant node generates signal across seam
    let mut const_node = DspNode::from_kind(ModuleKind::Constant, sample_rate, &[0.75]);
    let mut const_out = [0.0f32; 64];
    {
        let mut outputs = [&mut const_out[..]];
        const_node.process_block(&[], &mut outputs);
    }
    assert!(const_out.iter().all(|&x| (x - 0.75).abs() < f32::EPSILON));

    // 2. Oscillator node generates signal across seam
    let mut osc_node = DspNode::from_kind(ModuleKind::Oscillator, sample_rate, &[440.0]);
    let mut osc_out = [0.0f32; 64];
    {
        let mut outputs = [&mut osc_out[..]];
        osc_node.process_block(&[], &mut outputs);
    }
    let osc_max = osc_out.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(osc_max > 0.5);

    // 3. Gain node attenuates signal across seam
    let mut gain_node = DspNode::from_kind(ModuleKind::Gain, sample_rate, &[0.5]);
    let mut gain_out = [0.0f32; 64];
    {
        let inputs = [&const_out[..]];
        let mut outputs = [&mut gain_out[..]];
        gain_node.process_block(&inputs, &mut outputs);
    }
    let last_gain_sample = gain_out[63];
    assert!((last_gain_sample - 0.375).abs() < 0.1);

    // 4. Reset test
    osc_node.reset();
    gain_node.reset();
}
