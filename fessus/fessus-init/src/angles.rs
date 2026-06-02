/// Distributes N apps evenly along a 90° arc.
/// Returns a list of (angle_deg, index) pairs.
pub fn calculate(count: usize) -> Vec<f64> {
    match count {
        0 => vec![],
        1 => vec![45.0],
        n => (0..n)
            .map(|i| i as f64 * 90.0 / (n as f64 - 1.0))
            .collect(),
    }
}

/// Returns the CSS transform string for a pinned app at a given angle.
/// The counter-rotation keeps the icon upright along the arc.
pub fn css_transform(angle: f64, radius_rem: f64) -> String {
    format!(
        "rotate({angle}deg) translateX({radius}rem) rotate(-{angle}deg)",
        angle  = angle,
        radius = radius_rem,
    )
}
