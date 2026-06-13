use super::*;

#[test]
fn test_percentage() {
    assert_eq!(percentage(0, 0), 0.0);
    assert_eq!(percentage(5, 0), 0.0);
    assert_eq!(percentage(5, 10), 50.0);
    assert_eq!(percentage(10, 10), 100.0);
}

#[test]
fn test_lerp() {
    assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    // test clamping
    assert_eq!(lerp(0.0, 10.0, -0.5), 0.0);
    assert_eq!(lerp(0.0, 10.0, 1.5), 10.0);
}

#[test]
fn test_hsl_to_rgb() {
    // Red: HSL(0, 1, 0.5) -> RGB(255, 0, 0)
    let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 0);

    // Green: HSL(120, 1, 0.5) -> RGB(0, 255, 0)
    let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
    assert_eq!(r, 0);
    assert_eq!(g, 255);
    assert_eq!(b, 0);

    // Blue: HSL(240, 1, 0.5) -> RGB(0, 0, 255)
    let (r, g, b) = hsl_to_rgb(240.0, 1.0, 0.5);
    assert_eq!(r, 0);
    assert_eq!(g, 0);
    assert_eq!(b, 255);
}

#[test]
fn test_smooth_noise() {
    let n1 = smooth_noise(1.0, 0, 1.0, 1.0);
    let n2 = smooth_noise(1.0, 0, 1.0, 1.0);
    // Should be deterministic
    assert_eq!(n1, n2);

    let n3 = smooth_noise(2.0, 0, 1.0, 1.0);
    assert_ne!(n1, n3);
}

#[test]
fn test_percentage_precision() {
    assert_eq!(percentage(1, 3), (1.0 / 3.0) * 100.0);
    assert_eq!(percentage(12345, 12345), 100.0);
}

#[test]
fn test_lerp_extremes() {
    // factor > 1.0 should clamp to 1.0 (returning b)
    assert_eq!(lerp(5.0, 15.0, 2.5), 15.0);
    // factor < 0.0 should clamp to 0.0 (returning a)
    assert_eq!(lerp(5.0, 15.0, -1.2), 5.0);
}

#[test]
fn test_hsl_to_rgb_boundary_values() {
    // Test transitions near hue boundaries
    // Hue 59.0
    let (r, g, b) = hsl_to_rgb(59.0, 1.0, 0.5);
    assert!(r > 0);
    // Hue 119.0
    let (r, g, b) = hsl_to_rgb(119.0, 1.0, 0.5);
    assert!(g > 0);
    // Hue 179.0
    let (r, g, b) = hsl_to_rgb(179.0, 1.0, 0.5);
    assert!(g > 0 && b > 0);
    // Hue 239.0
    let (r, g, b) = hsl_to_rgb(239.0, 1.0, 0.5);
    assert!(b > 0);
    // Hue 299.0
    let (r, g, b) = hsl_to_rgb(299.0, 1.0, 0.5);
    assert!(r > 0 && b > 0);
    // Hue 350.0
    let (r, g, b) = hsl_to_rgb(350.0, 1.0, 0.5);
    assert!(r > 0);
}

#[test]
fn test_smooth_noise_amplitude_zero() {
    assert_eq!(smooth_noise(10.0, 2, 0.0, 1.5), 0.0);
    assert_eq!(smooth_noise(0.0, 0, 0.0, 0.0), 0.0);
}

#[test]
fn test_smooth_noise_frequency() {
    let n1 = smooth_noise(1.5, 3, 10.0, 0.5);
    let n2 = smooth_noise(1.5, 3, 10.0, 2.0);
    assert_ne!(n1, n2);
}

