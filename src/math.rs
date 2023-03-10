use crate::prelude::*;

pub fn random_point_in_circle(rng: &mut impl Rng, radius: f32) -> Vec2 {
    let t = 2.0 * PI * rng.gen_range(0.0..1.0);
    let u = rng.gen_range(0.0..2.0);
    let r = if u > 1.0 { 2.0 - u } else { u };
    Vec2::new(radius * r * t.cos(), radius * r * t.sin())
}

pub fn iter_float(range: RangeInclusive<f32>, step: f32) -> impl Iterator<Item = f32> {
    std::iter::successors(Some(*range.start()), move |i| {
        let next = *i + step;
        (next < *range.end()).then_some(next)
    })
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
