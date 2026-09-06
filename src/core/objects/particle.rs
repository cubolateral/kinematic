#[cfg(test)]
thread_local! {
    pub(crate) static CAPTURE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

use crate::core::{
    Easing,
    components::{PARTICLE_COUNT, PARTICLE_FADE_START, PARTICLE_RADIUS},
    objects::{MorphParticleRoute, draw_particle_batch, silhouette_grid},
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
}

impl Silhouette {
    pub(crate) fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Samples colors and positions from a drawing in local coordinates.
    pub(crate) fn capture(
        bounds: skia_safe::Rect,
        count: usize,
        draw: impl FnOnce(&skia_safe::Canvas),
    ) -> Self {
        #[cfg(test)]
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
            count,
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
    const SOURCE_FADE_END: f32 = 0.1;

    let (_, target_progress) = morph_opacities(progress);
    smoothstep((progress / SOURCE_FADE_END).clamp(0.0, 1.0)) * (1.0 - target_progress)
}

impl ParticleTransform {
    pub(crate) fn new(from: Silhouette, to: Silhouette, easing: Easing) -> Self {
        let count = if from.is_empty() || to.is_empty() {
            0
        } else {
            PARTICLE_COUNT as usize
        };
        let routes = (0..count)
            .map(|index| {
                MorphParticleRoute::new(
                    from.samples[index * from.samples.len() / count].point,
                    to.samples[index * to.samples.len() / count].point,
                    to.bounds,
                )
            })
            .collect();
        Self {
            from,
            to,
            easing,
            routes,
        }
    }

    pub(crate) fn draw(&self, canvas: &skia_safe::Canvas, progress: f32, opacity: f32) {
        let data = self;
        if data.from.samples.is_empty() || data.to.samples.is_empty() {
            return;
        }
        let particle_opacity = particle_opacity(progress);
        let count = data.routes.len();
        let mut positions = Vec::with_capacity(count);
        let mut colors = Vec::with_capacity(count);
        for (index, route) in data.routes.iter().enumerate() {
            let from = &data.from.samples[index * data.from.samples.len() / count];
            let to = &data.to.samples[index * data.to.samples.len() / count];
            let local = route.progress(progress);
            let t = data.easing.evaluate(local).clamp(0.0, 1.0);
            let point = route.position(t);
            let color: [f32; 4] =
                std::array::from_fn(|i| from.color[i] + (to.color[i] - from.color[i]) * t);
            positions.push(point);
            let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            colors.push(skia_safe::Color::from_argb(
                channel(color[3] * particle_opacity * opacity),
                channel(color[0]),
                channel(color[1]),
                channel(color[2]),
            ));
        }
        draw_particle_batch(canvas, &positions, &colors, PARTICLE_RADIUS);
    }
}
