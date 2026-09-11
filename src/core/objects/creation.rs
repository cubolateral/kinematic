use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::{
    components::{
        Morph, PARTICLE_DISTANCE, PARTICLE_FADE_START, PARTICLE_RADIUS, PARTICLE_STAGGER, Style,
    },
    types::{Color, Vector2},
};

const ATLAS_PADDING: f32 = 2.0;
pub(crate) const OBJECT_FADE_START: f32 = PARTICLE_FADE_START;
const MAX_ATLAS_DIMENSION: f32 = 2048.0;
const MAX_CACHE_ENTRIES: usize = 64;
const MAX_GRID_OVERSAMPLING: usize = 64;
const MAX_PARTICLE_COUNT: usize = 20_000;
const PARTICLE_SPACING: f32 = 2.0;
const PARTICLE_SPRITE_RADIUS: f32 = 14.0;
const PARTICLE_SPRITE_SIZE: i32 = 32;

thread_local! {
    static PARTICLE_CACHE: RefCell<HashMap<u64, CachedParticles>> = RefCell::new(HashMap::new());
    static PARTICLE_SPRITE: skia_safe::Image = particle_sprite();
}

struct Particle {
    target: Vector2,
    start_offset: Vector2,
    control_1_offset: Vector2,
    control_2_offset: Vector2,
    start: f32,
}

struct CachedParticles {
    fingerprint: u64,
    particles: Vec<Particle>,
    transforms: Vec<skia_safe::RSXform>,
    sources: Vec<skia_safe::Rect>,
    colors: Vec<skia_safe::Color>,
}

/// Produces a cache key for the silhouette of an object.
pub(crate) fn particle_visual_key(
    kind: &str,
    style: &Style,
    numbers: &[f32],
    strings: &[&str],
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    kind.hash(&mut hasher);

    (style.fill.a > 0.0).hash(&mut hasher);
    (style.stroke.a > 0.0).hash(&mut hasher);
    style.stroke_width.to_bits().hash(&mut hasher);

    for value in numbers {
        value.to_bits().hash(&mut hasher);
    }

    for value in strings {
        value.hash(&mut hasher);
    }

    hasher.finish()
}

/// Parameters for one silhouette-forming particle draw.
pub(crate) struct CreationDraw<'a> {
    pub entity: hecs::Entity,
    pub cache_slot: u64,
    pub bounds: skia_safe::Rect,
    pub visual_key: u64,
    pub style: &'a Style,
    pub pixel_color: Option<&'a dyn Fn(Vector2) -> Color>,
    pub morph: &'a Morph,
    pub opacity: f32,
    pub canvas: &'a skia_safe::Canvas,
}

impl CreationDraw<'_> {
    /// Draws the particle cloud in one batched atlas call.
    pub fn render(self, draw_complete: impl Fn(&skia_safe::Canvas, f32)) -> bool {
        let Self {
            entity,
            cache_slot,
            bounds,
            visual_key,
            style,
            pixel_color,
            morph,
            opacity,
            canvas,
        } = self;

        if !morph.particles_enabled || morph.progress >= 1.0 {
            return false;
        }

        let progress = morph.progress.clamp(0.0, 1.0);
        if progress <= 0.0 {
            return true;
        }
        if !valid_bounds(bounds) {
            return false;
        }

        let density = mask_density(canvas, bounds);
        let particle_count = particle_count_for_bounds(bounds);
        let fingerprint = cache_fingerprint(visual_key, bounds, density, particle_count);
        let entity_key = entity
            .to_bits()
            .get()
            .wrapping_add(cache_slot.rotate_left(32));
        let rendered = PARTICLE_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let rebuild = cache
                .get(&entity_key)
                .is_none_or(|cached| cached.fingerprint != fingerprint);

            if rebuild {
                let Some(cached) =
                    build_cache(bounds, density, fingerprint, particle_count, &draw_complete)
                else {
                    return false;
                };

                if cache.len() >= MAX_CACHE_ENTRIES && !cache.contains_key(&entity_key) {
                    cache.clear();
                }
                cache.insert(entity_key, cached);
            }

            let cached = cache.get_mut(&entity_key).unwrap();
            draw_particles(cached, progress, style, pixel_color, opacity, canvas);
            true
        });

        if rendered {
            let completion = completion_progress(progress);
            if completion > 0.0 {
                draw_complete(canvas, opacity * completion);
            }
        }

        rendered
    }
}

fn valid_bounds(bounds: skia_safe::Rect) -> bool {
    bounds.left.is_finite()
        && bounds.top.is_finite()
        && bounds.right.is_finite()
        && bounds.bottom.is_finite()
        && bounds.width() > 0.0
        && bounds.height() > 0.0
}

/// Keeps particle centers two pixels apart across an object's bounds.
pub(crate) fn particle_count_for_bounds(bounds: skia_safe::Rect) -> usize {
    if !valid_bounds(bounds) {
        return 0;
    }

    let columns = (bounds.width() / PARTICLE_SPACING).ceil() as usize;
    let rows = (bounds.height() / PARTICLE_SPACING).ceil() as usize;
    columns.saturating_mul(rows).clamp(1, MAX_PARTICLE_COUNT)
}

fn mask_density(canvas: &skia_safe::Canvas, bounds: skia_safe::Rect) -> f32 {
    let matrix = canvas.local_to_device_as_3x3();
    let scale_x = matrix.scale_x().hypot(matrix.skew_y());
    let scale_y = matrix.skew_x().hypot(matrix.scale_y());
    let desired = ((scale_x.max(scale_y).max(1.0) * 2.0).ceil() * 0.5).min(2.0);
    (MAX_ATLAS_DIMENSION / bounds.width().max(bounds.height())).min(desired)
}

fn cache_fingerprint(
    visual_key: u64,
    bounds: skia_safe::Rect,
    density: f32,
    particle_count: usize,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    visual_key.hash(&mut hasher);
    bounds.left.to_bits().hash(&mut hasher);
    bounds.top.to_bits().hash(&mut hasher);
    bounds.right.to_bits().hash(&mut hasher);
    bounds.bottom.to_bits().hash(&mut hasher);
    density.to_bits().hash(&mut hasher);
    particle_count.hash(&mut hasher);
    hasher.finish()
}

fn build_cache(
    bounds: skia_safe::Rect,
    density: f32,
    fingerprint: u64,
    particle_count: usize,
    draw_complete: &impl Fn(&skia_safe::Canvas, f32),
) -> Option<CachedParticles> {
    let origin = Vector2::new(
        bounds.left - ATLAS_PADDING / density,
        bounds.top - ATLAS_PADDING / density,
    );
    let width = (bounds.width() * density).ceil() as i32 + (ATLAS_PADDING as i32 * 2);
    let height = (bounds.height() * density).ceil() as i32 + (ATLAS_PADDING as i32 * 2);
    let mut surface = skia_safe::surfaces::raster_n32_premul((width.max(1), height.max(1)))?;
    let mask_canvas = surface.canvas();
    mask_canvas.clear(skia_safe::colors::TRANSPARENT);
    mask_canvas.scale((density, density));
    mask_canvas.translate((-origin.x, -origin.y));
    draw_complete(mask_canvas, 1.0);

    let particle_count = particle_count.clamp(1, MAX_PARTICLE_COUNT);
    let targets = silhouette_grid(&mut surface, particle_count, origin, density, bounds);
    let mut particles: Vec<_> = targets
        .into_iter()
        .map(|target| particle_route(target, bounds))
        .collect();
    particles.sort_unstable_by(|left, right| left.start.total_cmp(&right.start));
    let count = particles.len();
    let source = skia_safe::Rect::from_wh(PARTICLE_SPRITE_SIZE as f32, PARTICLE_SPRITE_SIZE as f32);

    Some(CachedParticles {
        fingerprint,
        particles,
        transforms: vec![skia_safe::RSXform::new(1.0, 0.0, (0.0, 0.0)); count],
        sources: vec![source; count],
        colors: vec![skia_safe::Color::TRANSPARENT; count],
    })
}

pub(crate) fn silhouette_grid(
    surface: &mut skia_safe::Surface,
    count: usize,
    origin: Vector2,
    density: f32,
    bounds: skia_safe::Rect,
) -> Vec<Vector2> {
    let pixels = surface.peek_pixels().unwrap();
    let size = pixels.dimensions();
    if count == 0 {
        return Vec::new();
    }
    // Coverage is only a density estimate; bound its cost independently of capture resolution.
    let step_x = (size.width / 128).max(1) as usize;
    let step_y = (size.height / 128).max(1) as usize;
    let mut occupied = 0usize;
    let mut sampled = 0usize;
    for y in (0..size.height).step_by(step_y) {
        for x in (0..size.width).step_by(step_x) {
            occupied += usize::from(pixels.get_color((x, y)).a() > 0);
            sampled += 1;
        }
    }
    // A thin silhouette can fall between probes. Let the grid refinement find it.
    let coverage =
        (occupied as f32 / sampled.max(1) as f32).max(1.0 / MAX_GRID_OVERSAMPLING as f32);
    let maximum_cells = count.saturating_mul(MAX_GRID_OVERSAMPLING);
    let mut cell_count = ((count as f32 / coverage).ceil() as usize).clamp(count, maximum_cells);
    let mut samples = Vec::with_capacity(count);

    for _ in 0..3 {
        let spacing = grid_spacing(bounds, cell_count);
        samples.clear();
        collect_grid_targets(
            &mut samples,
            &pixels,
            size,
            origin,
            density,
            bounds,
            spacing,
        );

        if samples.len() >= count || cell_count == maximum_cells {
            break;
        }

        let correction = count as f32 / samples.len().max(1) as f32;
        let next = ((cell_count as f32 * correction * 1.05).ceil() as usize)
            .clamp(cell_count + 1, maximum_cells);
        cell_count = next;
    }

    if samples.len() > count {
        let total = samples.len();
        for index in 0..count {
            samples[index] = samples[index * total / count];
        }
        samples.truncate(count);
    }

    samples
}

fn grid_spacing(bounds: skia_safe::Rect, cell_count: usize) -> f32 {
    let aspect = bounds.width() / bounds.height();
    let columns = ((cell_count as f32 * aspect).sqrt().ceil() as usize).max(1);
    let rows = cell_count.div_ceil(columns).max(1);

    (bounds.width() / columns as f32).max(bounds.height() / rows as f32)
}

fn collect_grid_targets(
    samples: &mut Vec<Vector2>,
    pixels: &skia_safe::Pixmap<'_>,
    size: skia_safe::ISize,
    origin: Vector2,
    density: f32,
    bounds: skia_safe::Rect,
    spacing: f32,
) {
    let mut y = bounds.top + spacing * 0.5;

    while y < bounds.bottom {
        let mut x = bounds.left + spacing * 0.5;

        while x < bounds.right {
            let pixel_x = ((x - origin.x) * density).floor() as i32;
            let pixel_y = ((y - origin.y) * density).floor() as i32;
            let inside_mask = pixel_x >= 0
                && pixel_y >= 0
                && pixel_x < size.width
                && pixel_y < size.height
                && pixels.get_color((pixel_x, pixel_y)).a() > 0;

            if inside_mask {
                samples.push(Vector2::new(x, y));
            }

            x += spacing;
        }

        y += spacing;
    }
}

fn draw_particles(
    cached: &mut CachedParticles,
    progress: f32,
    style: &Style,
    pixel_color: Option<&dyn Fn(Vector2) -> Color>,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    if cached.particles.is_empty() || PARTICLE_RADIUS <= 0.0 {
        return;
    }

    let stagger = PARTICLE_STAGGER;
    let distance = PARTICLE_DISTANCE;
    let cloud_opacity = 1.0 - completion_progress(progress);
    let particle_progress = progress;
    let particle_color = if style.fill.a > 0.0 {
        style.fill
    } else {
        style.stroke
    };
    let radius = PARTICLE_RADIUS;
    let scale = radius / PARTICLE_SPRITE_RADIUS;
    let active_count = cached
        .particles
        .partition_point(|particle| particle.start * stagger <= particle_progress);

    if active_count == 0 {
        return;
    }

    for (index, particle) in cached.particles[..active_count].iter().enumerate() {
        let local = local_particle_progress(particle_progress, particle.start, stagger);
        let fade = smoothstep((local / 0.25).min(1.0)) * cloud_opacity;
        let position = particle_position(particle, distance, local);

        cached.transforms[index] = skia_safe::RSXform::from_radians(
            scale,
            0.0,
            (position.x, position.y),
            (
                PARTICLE_SPRITE_SIZE as f32 * 0.5,
                PARTICLE_SPRITE_SIZE as f32 * 0.5,
            ),
        );
        cached.colors[index] = skia_color(
            pixel_color
                .map(|get_pixel_color| get_pixel_color(particle.target))
                .unwrap_or(particle_color),
            fade,
        );
    }

    super::particle::DRAW_COUNT.set(super::particle::DRAW_COUNT.get() + active_count);
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_alpha_f(opacity.clamp(0.0, 1.0));
    PARTICLE_SPRITE.with(|sprite| {
        canvas.draw_atlas(
            sprite,
            &cached.transforms[..active_count],
            &cached.sources[..active_count],
            Some(&cached.colors[..active_count]),
            skia_safe::BlendMode::Modulate,
            skia_safe::FilterMode::Nearest,
            None,
            &paint,
        );
    });
}

pub(crate) struct ParticleBatch {
    transforms: Vec<skia_safe::RSXform>,
    sources: Vec<skia_safe::Rect>,
    colors: Vec<skia_safe::Color>,
}

impl ParticleBatch {
    pub fn new(count: usize) -> Self {
        Self {
            transforms: vec![skia_safe::RSXform::new(1.0, 0.0, (0.0, 0.0)); count],
            sources: vec![
                skia_safe::Rect::from_wh(
                    PARTICLE_SPRITE_SIZE as f32,
                    PARTICLE_SPRITE_SIZE as f32
                );
                count
            ],
            colors: vec![skia_safe::Color::TRANSPARENT; count],
        }
    }

    pub fn draw(
        &mut self,
        canvas: &skia_safe::Canvas,
        radius: f32,
        mut particle: impl FnMut(usize) -> (Vector2, skia_safe::Color),
    ) {
        if radius <= 0.0 || self.transforms.is_empty() {
            return;
        }
        super::particle::DRAW_COUNT.set(super::particle::DRAW_COUNT.get() + self.transforms.len());
        let scale = radius / PARTICLE_SPRITE_RADIUS;
        let offset = PARTICLE_SPRITE_SIZE as f32 * 0.5 * scale;
        for index in 0..self.transforms.len() {
            let (position, color) = particle(index);
            self.transforms[index] =
                skia_safe::RSXform::new(scale, 0.0, (position.x - offset, position.y - offset));
            self.colors[index] = color;
        }
        PARTICLE_SPRITE.with(|sprite| {
            canvas.draw_atlas(
                sprite,
                &self.transforms,
                &self.sources,
                Some(self.colors.as_slice()),
                skia_safe::BlendMode::Modulate,
                skia_safe::FilterMode::Nearest,
                None,
                &skia_safe::Paint::default(),
            );
        });
    }
}

fn particle_route(target: Vector2, bounds: skia_safe::Rect) -> Particle {
    let center = Vector2::new(bounds.center_x(), bounds.center_y());
    let half_size = Vector2::new(bounds.width() * 0.5, bounds.height() * 0.5);
    let normalized = (target - center) / half_size.max(Vector2::splat(0.001));
    let noise_position = (normalized + Vector2::ONE) * 2.25;
    let noise = coherent_noise(noise_position, 0);
    let detail = coherent_noise(noise_position * 2.0 + Vector2::splat(7.31), 1);
    let epsilon = 0.08;
    let gradient_x = coherent_noise(noise_position + Vector2::X * epsilon, 2)
        - coherent_noise(noise_position - Vector2::X * epsilon, 2);
    let gradient_y = coherent_noise(noise_position + Vector2::Y * epsilon, 2)
        - coherent_noise(noise_position - Vector2::Y * epsilon, 2);
    let flow = Vector2::new(gradient_y, -gradient_x).normalize_or_zero();
    let fallback = (target - center).perp().normalize_or_zero();
    let flow = if flow.length_squared() > 0.001 {
        flow
    } else {
        fallback
    };
    let jitter = hash_direction(target, bounds);
    let direction = (flow * 0.82 + jitter * 0.18).normalize_or_zero();
    let bend_sign = if detail >= 0.5 { 1.0 } else { -1.0 };
    let bend = direction.perp() * bend_sign * (0.22 + noise * 0.24);
    let reach = 0.72 + detail * 0.48;
    let radius = (normalized.length() / std::f32::consts::SQRT_2).min(1.0);
    let start = (radius * 0.72 + noise * 0.28 - 0.12).clamp(0.0, 1.0);

    Particle {
        target,
        start_offset: direction * reach,
        control_1_offset: direction * reach + bend,
        control_2_offset: direction * 0.18 - bend * 0.35,
        start,
    }
}

fn particle_position(particle: &Particle, distance: f32, progress: f32) -> Vector2 {
    if distance <= 0.0 {
        return particle.target;
    }

    cubic_bezier(
        particle.target + particle.start_offset * distance,
        particle.target + particle.control_1_offset * distance,
        particle.target + particle.control_2_offset * distance,
        particle.target,
        smoothstep(progress),
    )
}

/// Precomputed path and delay for one particle between two silhouettes.
pub(crate) struct MorphParticleRoute {
    coefficients: [Vector2; 4],
    to: Vector2,
    start: f32,
    inverse_duration: f32,
}

impl MorphParticleRoute {
    pub(crate) fn new(from: Vector2, to: Vector2, bounds: skia_safe::Rect) -> Self {
        let route = particle_route(to, bounds);
        let c1 = from + route.control_1_offset * PARTICLE_DISTANCE;
        let c2 = to + route.control_2_offset * PARTICLE_DISTANCE;
        let start = route.start * PARTICLE_STAGGER;
        Self {
            coefficients: [
                to - c2 * 3.0 + c1 * 3.0 - from,
                (c2 - c1 * 2.0 + from) * 3.0,
                (c1 - from) * 3.0,
                from,
            ],
            to,
            start,
            inverse_duration: 1.0 / (1.0 - start),
        }
    }

    pub(crate) fn position(&self, progress: f32) -> Vector2 {
        if progress == 1.0 {
            return self.to;
        }
        let t = smoothstep(progress);
        let [a, b, c, d] = self.coefficients;
        ((a * t + b) * t + c) * t + d
    }

    pub(crate) fn progress(&self, progress: f32) -> f32 {
        ((progress - self.start) * self.inverse_duration).clamp(0.0, 1.0)
    }
}

fn coherent_noise(position: Vector2, salt: u32) -> f32 {
    let cell_x = position.x.floor() as i32;
    let cell_y = position.y.floor() as i32;
    let fraction = position - Vector2::new(cell_x as f32, cell_y as f32);
    let blend = fraction * fraction * (Vector2::splat(3.0) - fraction * 2.0);
    let top = lerp(
        lattice_noise(cell_x, cell_y, salt),
        lattice_noise(cell_x + 1, cell_y, salt),
        blend.x,
    );
    let bottom = lerp(
        lattice_noise(cell_x, cell_y + 1, salt),
        lattice_noise(cell_x + 1, cell_y + 1, salt),
        blend.x,
    );

    lerp(top, bottom, blend.y)
}

fn lattice_noise(x: i32, y: i32, salt: u32) -> f32 {
    let mut value = (x as u32)
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add((y as u32).wrapping_mul(0x85eb_ca6b))
        .wrapping_add(salt.wrapping_mul(0xc2b2_ae35));
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;

    value as f32 / u32::MAX as f32
}

fn hash_direction(target: Vector2, bounds: skia_safe::Rect) -> Vector2 {
    let x = ((target.x - bounds.left) * 100.0).round() as i32;
    let y = ((target.y - bounds.top) * 100.0).round() as i32;
    let angle = lattice_noise(x, y, 3) * std::f32::consts::TAU;

    Vector2::new(angle.cos(), angle.sin())
}

fn lerp(start: f32, end: f32, progress: f32) -> f32 {
    start + (end - start) * progress
}

fn cubic_bezier(
    start: Vector2,
    control_1: Vector2,
    control_2: Vector2,
    end: Vector2,
    progress: f32,
) -> Vector2 {
    let remaining = 1.0 - progress;

    start * remaining.powi(3)
        + control_1 * (3.0 * remaining.powi(2) * progress)
        + control_2 * (3.0 * remaining * progress.powi(2))
        + end * progress.powi(3)
}

fn particle_sprite() -> skia_safe::Image {
    let mut surface =
        skia_safe::surfaces::raster_n32_premul((PARTICLE_SPRITE_SIZE, PARTICLE_SPRITE_SIZE))
            .unwrap();
    let canvas = surface.canvas();
    canvas.clear(skia_safe::colors::TRANSPARENT);

    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(skia_safe::Color::WHITE);
    canvas.draw_circle(
        (
            PARTICLE_SPRITE_SIZE as f32 * 0.5,
            PARTICLE_SPRITE_SIZE as f32 * 0.5,
        ),
        PARTICLE_SPRITE_RADIUS,
        &paint,
    );
    surface.image_snapshot()
}

fn skia_color(color: Color, opacity: f32) -> skia_safe::Color {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

    skia_safe::Color::from_argb(
        channel(color.a * opacity),
        channel(color.r),
        channel(color.g),
        channel(color.b),
    )
}

fn completion_progress(progress: f32) -> f32 {
    smoothstep(((progress - OBJECT_FADE_START) / (1.0 - OBJECT_FADE_START)).clamp(0.0, 1.0))
}

fn local_particle_progress(progress: f32, start: f32, stagger: f32) -> f32 {
    let start = start * stagger;

    if start >= 1.0 {
        return if progress >= 1.0 { 1.0 } else { 0.0 };
    }

    ((progress - start) / (1.0 - start)).clamp(0.0, 1.0)
}

fn smoothstep(progress: f32) -> f32 {
    progress * progress * (3.0 - 2.0 * progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organized_chaos_curves_and_settles_on_its_grid_cell() {
        let bounds = skia_safe::Rect::from_wh(100.0, 100.0);
        let target = Vector2::new(20.0, 10.0);
        let particle = particle_route(target, bounds);
        let start = particle_position(&particle, 100.0, 0.0);
        let middle = particle_position(&particle, 100.0, 0.5);
        let linear_middle = start.lerp(target, 0.5);

        assert_ne!(start, target);
        assert!(middle.distance(linear_middle) > 0.01);
        assert_eq!(particle_position(&particle, 100.0, 1.0), target);
        assert_eq!(particle_position(&particle, 0.0, 0.5), target);
    }

    #[test]
    fn particle_count_uses_a_two_pixel_grid() {
        assert_eq!(
            particle_count_for_bounds(skia_safe::Rect::from_wh(10.0, 20.0)),
            50
        );
        assert_eq!(
            particle_count_for_bounds(skia_safe::Rect::from_wh(11.0, 21.0)),
            66
        );
        assert_eq!(
            particle_count_for_bounds(skia_safe::Rect::from_wh(1_000.0, 1_000.0)),
            MAX_PARTICLE_COUNT
        );
    }

    #[test]
    fn organized_chaos_builds_center_before_outer_regions() {
        let bounds = skia_safe::Rect::from_wh(100.0, 100.0);
        let center = particle_route(Vector2::new(50.0, 50.0), bounds);
        let corner = particle_route(Vector2::new(100.0, 100.0), bounds);

        assert!(center.start < corner.start);
    }

    #[test]
    fn organized_chaos_is_deterministic() {
        let bounds = skia_safe::Rect::from_wh(100.0, 100.0);
        let first = particle_route(Vector2::new(25.0, 75.0), bounds);
        let second = particle_route(Vector2::new(25.0, 75.0), bounds);

        assert_eq!(first.start_offset, second.start_offset);
        assert_eq!(first.control_1_offset, second.control_1_offset);
        assert_eq!(first.control_2_offset, second.control_2_offset);
        assert_eq!(first.start, second.start);
    }

    #[test]
    fn object_fade_uses_the_configured_completion_range() {
        assert_eq!(completion_progress(0.0), 0.0);
        assert_eq!(completion_progress(OBJECT_FADE_START), 0.0);
        let midpoint = OBJECT_FADE_START + (1.0 - OBJECT_FADE_START) * 0.5;
        assert!((completion_progress(midpoint) - 0.5).abs() < 0.00001);
        assert_eq!(completion_progress(1.0), 1.0);
    }

    #[test]
    fn particles_keep_organizing_during_the_object_fade() {
        assert_eq!(local_particle_progress(0.75, 0.0, 0.0), 0.75);
        assert_eq!(local_particle_progress(0.75, 1.0, 0.95), 0.0);
        assert_eq!(local_particle_progress(1.0, 1.0, 0.95), 1.0);
    }

    #[test]
    fn silhouette_targets_share_a_perfect_square_grid() {
        let bounds = skia_safe::Rect::from_wh(20.0, 10.0);
        let mut surface = skia_safe::surfaces::raster_n32_premul((20, 10)).unwrap();
        surface.canvas().clear(skia_safe::colors::WHITE);
        let count = 20_usize;
        let targets = silhouette_grid(&mut surface, count, Vector2::ZERO, 1.0, bounds);
        let spacing = targets[1].x - targets[0].x;
        let grid_origin = targets[0];

        assert!(spacing > 0.0);
        for target in targets {
            let column = (target.x - grid_origin.x) / spacing;
            let row = (target.y - grid_origin.y) / spacing;

            assert!((column - column.round()).abs() < 0.001);
            assert!((row - row.round()).abs() < 0.001);
        }
    }

    #[test]
    fn sparse_silhouettes_still_receive_the_requested_particle_density() {
        let bounds = skia_safe::Rect::from_wh(100.0, 20.0);
        let mut surface = skia_safe::surfaces::raster_n32_premul((100, 20)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        let mut paint = skia_safe::Paint::default();
        paint.set_color(skia_safe::Color::WHITE);
        surface
            .canvas()
            .draw_rect(skia_safe::Rect::from_wh(20.0, 20.0), &paint);

        let targets = silhouette_grid(&mut surface, 100, Vector2::ZERO, 1.0, bounds);

        assert_eq!(targets.len(), 100);
    }
}
