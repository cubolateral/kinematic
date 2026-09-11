thread_local! {
    pub(crate) static DRAW_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static CAPTURE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

use crate::core::{
    Easing,
    components::{PARTICLE_FADE_START, PARTICLE_RADIUS},
    objects::{MorphParticleRoute, ParticleBatch, particle_count_for_bounds, silhouette_grid},
    types::Vector2,
};

struct Sample {
    point: Vector2,
    color: [f32; 4],
}

pub(crate) struct Silhouette {
    pub(crate) bounds: skia_safe::Rect,
    samples: Vec<Sample>,
}

pub(crate) struct ParticleTransform {
    pub(crate) from: Silhouette,
    pub(crate) to: Silhouette,
    easing: Easing,
    routes: Vec<MorphParticleRoute>,
    colors: Vec<([f32; 4], [f32; 4])>,
    batch: std::sync::Mutex<ParticleBatch>,
}

impl Silhouette {
    pub(crate) fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn sample_y_span(&self) -> f32 {
        let minimum = self
            .samples
            .iter()
            .map(|sample| sample.point.y)
            .fold(f32::INFINITY, f32::min);
        let maximum = self
            .samples
            .iter()
            .map(|sample| sample.point.y)
            .fold(f32::NEG_INFINITY, f32::max);

        maximum - minimum
    }

    #[cfg(test)]
    pub(crate) fn sample_x_span(&self) -> f32 {
        let minimum = self
            .samples
            .iter()
            .map(|sample| sample.point.x)
            .fold(f32::INFINITY, f32::min);
        let maximum = self
            .samples
            .iter()
            .map(|sample| sample.point.x)
            .fold(f32::NEG_INFINITY, f32::max);

        maximum - minimum
    }

    /// Samples colors and positions from a drawing in local coordinates.
    pub(crate) fn capture(bounds: skia_safe::Rect, draw: impl FnOnce(&skia_safe::Canvas)) -> Self {
        CAPTURE_COUNT.set(CAPTURE_COUNT.get() + 1);
        assert!(
            bounds.left.is_finite()
                && bounds.top.is_finite()
                && bounds.right.is_finite()
                && bounds.bottom.is_finite(),
            "Particle bounds must be finite."
        );
        let density = (2048.0 / bounds.width().max(bounds.height()).max(1.0)).min(2.0);
        let dimensions = (
            (bounds.width() * density).ceil().max(1.0) as i32,
            (bounds.height() * density).ceil().max(1.0) as i32,
        );
        let mut surface = skia_safe::surfaces::raster_n32_premul(dimensions)
            .expect("Morph silhouette allocation failed.");
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().scale((density, density));
        surface.canvas().translate((-bounds.left, -bounds.top));
        draw(surface.canvas());
        let points = silhouette_grid(
            &mut surface,
            particle_count_for_bounds(bounds),
            Vector2::new(bounds.left, bounds.top),
            density,
            bounds,
        );
        let samples = {
            let pixels = surface.peek_pixels().unwrap();
            points
                .into_iter()
                .map(|point| {
                    let color = pixels.get_color((
                        ((point.x - bounds.left) * density) as i32,
                        ((point.y - bounds.top) * density) as i32,
                    ));
                    Sample {
                        point,
                        color: [
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ],
                    }
                })
                .collect::<Vec<_>>()
        };
        Silhouette { bounds, samples }
    }

    pub(crate) fn collapsed_at(&self, anchors: &[Vector2]) -> Self {
        let fallback = [Vector2::ZERO];
        let anchors = if anchors.is_empty() {
            fallback.as_slice()
        } else {
            anchors
        };
        let samples = self
            .samples
            .iter()
            .map(|sample| {
                let point = anchors
                    .iter()
                    .copied()
                    .min_by(|left, right| {
                        sample
                            .point
                            .distance_squared(*left)
                            .total_cmp(&sample.point.distance_squared(*right))
                    })
                    .unwrap();
                Sample {
                    point,
                    color: sample.color,
                }
            })
            .collect();

        Self {
            bounds: self.bounds,
            samples,
        }
    }
}

fn smoothstep(progress: f32) -> f32 {
    progress * progress * (3.0 - 2.0 * progress)
}

pub(crate) fn morph_opacities(progress: f32) -> (f32, f32) {
    let fade_duration = 1.0 - PARTICLE_FADE_START;
    (
        1.0 - smoothstep((progress / fade_duration).clamp(0.0, 1.0)),
        smoothstep(((progress - PARTICLE_FADE_START) / fade_duration).clamp(0.0, 1.0)),
    )
}

fn particle_opacity(progress: f32) -> f32 {
    let fade_duration = 1.0 - PARTICLE_FADE_START;
    let (_, target_progress) = morph_opacities(progress);
    smoothstep((progress / fade_duration).clamp(0.0, 1.0)) * (1.0 - target_progress)
}

impl ParticleTransform {
    pub(crate) fn new(from: Silhouette, to: Silhouette, easing: Easing) -> Self {
        Self::sampled(from, to, easing)
    }

    pub(crate) fn sampled(from: Silhouette, to: Silhouette, easing: Easing) -> Self {
        let count = if from.is_empty() || to.is_empty() {
            0
        } else {
            from.samples.len().max(to.samples.len())
        };
        Self::with_count(from, to, easing, count)
    }

    fn with_count(from: Silhouette, to: Silhouette, easing: Easing, count: usize) -> Self {
        let routes = Self::routes(&from, &to, count);
        let colors = Self::colors(&from, &to, count);
        Self {
            from,
            to,
            easing,
            routes,
            colors,
            batch: std::sync::Mutex::new(ParticleBatch::new(count)),
        }
    }

    fn colors(from: &Silhouette, to: &Silhouette, count: usize) -> Vec<([f32; 4], [f32; 4])> {
        (0..count)
            .map(|index| {
                let from = from.samples[index * from.samples.len() / count].color;
                let to = to.samples[index * to.samples.len() / count].color;
                (from, std::array::from_fn(|i| to[i] - from[i]))
            })
            .collect()
    }

    fn routes(from: &Silhouette, to: &Silhouette, count: usize) -> Vec<MorphParticleRoute> {
        (0..count)
            .map(|index| {
                MorphParticleRoute::new(
                    from.samples[index * from.samples.len() / count].point,
                    to.samples[index * to.samples.len() / count].point,
                    to.bounds,
                )
            })
            .collect()
    }

    pub(crate) fn rebuild_routes(&mut self) {
        let count = if self.from.is_empty() || self.to.is_empty() {
            0
        } else {
            self.from.samples.len().max(self.to.samples.len())
        };
        self.routes = Self::routes(&self.from, &self.to, count);
        self.colors = Self::colors(&self.from, &self.to, count);
        *self.batch.get_mut().unwrap() = ParticleBatch::new(count);
    }

    pub(crate) fn draw(&self, canvas: &skia_safe::Canvas, progress: f32, opacity: f32) {
        let data = self;
        if data.from.samples.is_empty() || data.to.samples.is_empty() {
            return;
        }
        let particle_opacity = particle_opacity(progress);
        if particle_opacity * opacity <= 0.0 {
            return;
        }
        self.batch
            .lock()
            .unwrap()
            .draw(canvas, PARTICLE_RADIUS, |index| {
                let route = &data.routes[index];
                let t = data
                    .easing
                    .evaluate(route.progress(progress))
                    .clamp(0.0, 1.0);
                let point = route.position(t);
                let (from, delta) = data.colors[index];
                let color: [f32; 4] = std::array::from_fn(|i| from[i] + delta[i] * t);
                let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                (
                    point,
                    skia_safe::Color::from_argb(
                        channel(color[3] * particle_opacity * opacity),
                        channel(color[0]),
                        channel(color[1]),
                        channel(color[2]),
                    ),
                )
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_fades_only_near_its_endpoints() {
        assert_eq!(morph_opacities(0.0), (1.0, 0.0));
        assert_eq!(morph_opacities(0.5), (0.0, 0.0));
        assert_eq!(morph_opacities(1.0), (0.0, 1.0));
        assert_eq!(particle_opacity(0.0), 0.0);
        assert!(particle_opacity(0.01) > 0.0);
        assert_eq!(particle_opacity(0.5), 1.0);
    }
}
