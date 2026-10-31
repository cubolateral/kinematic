use serde::{Deserialize, Serialize};

use crate::core::ProjectContext;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) enum MarginUnit {
    #[default]
    Percentage,
    Pixels,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(crate) struct Margin {
    pub(crate) unit: MarginUnit,
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) bottom: f32,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            unit: MarginUnit::Percentage,
            left: 5.0,
            right: 5.0,
            top: 5.0,
            bottom: 5.0,
        }
    }
}

impl Margin {
    pub(crate) fn pixels(self, resolution: (u32, u32)) -> [f32; 4] {
        let [x, y] = match self.unit {
            MarginUnit::Percentage => [resolution.0 as f32 / 100.0, resolution.1 as f32 / 100.0],
            MarginUnit::Pixels => [1.0, 1.0],
        };
        [self.left * x, self.right * x, self.top * y, self.bottom * y]
    }

    pub(crate) fn is_valid(self, resolution: (u32, u32)) -> bool {
        let sides = self.pixels(resolution);
        sides.iter().all(|v| v.is_finite() && *v >= 0.0)
            && sides[0] + sides[1] < resolution.0 as f32
            && sides[2] + sides[3] < resolution.1 as f32
    }
}

impl<'de> Deserialize<'de> for Margin {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Margin;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("Margin settings or a legacy pixel pair.")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Margin, A::Error> {
                let x: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let y: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::invalid_length(3, &self));
                }
                Ok(Margin {
                    unit: MarginUnit::Pixels,
                    left: x as f32,
                    right: x as f32,
                    top: y as f32,
                    bottom: y as f32,
                })
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Margin, A::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields {
                    unit: MarginUnit,
                    left: f32,
                    right: f32,
                    top: f32,
                    bottom: f32,
                }
                let fields =
                    Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(Margin {
                    unit: fields.unit,
                    left: fields.left,
                    right: fields.right,
                    top: fields.top,
                    bottom: fields.bottom,
                })
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct ProjectSettings {
    pub(crate) resolution: (u32, u32),
    pub(crate) fps: u32,
    pub(crate) margin: Margin,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            fps: 60,
            margin: Margin::default(),
        }
    }
}

impl ProjectSettings {
    pub(crate) fn is_valid(settings: &Self) -> bool {
        settings.resolution.0 > 0
            && settings.resolution.1 > 0
            && settings.resolution.0.is_multiple_of(2)
            && settings.resolution.1.is_multiple_of(2)
            && settings.fps > 0
            && settings.resolution.0 <= 8192
            && settings.resolution.1 <= 8192
            && settings.fps <= 1000
            && settings.margin.is_valid(settings.resolution)
    }
}

pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) context: ProjectContext,
    pub(crate) settings: ProjectSettings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margins_preserve_legacy_pixels_and_round_trip_each_side() {
        let legacy: Margin = ron::from_str("(64, 32)").unwrap();
        assert_eq!(legacy.unit, MarginUnit::Pixels);
        assert_eq!(legacy.pixels((1280, 720)), [64.0, 64.0, 32.0, 32.0]);
        assert_eq!(
            Margin::default().pixels((1280, 720)),
            [64.0, 64.0, 36.0, 36.0]
        );
        let margin = Margin {
            left: 10.0,
            right: 20.0,
            top: 5.0,
            bottom: 15.0,
            ..Margin::default()
        };
        let text = ron::to_string(&margin).unwrap();
        assert_eq!(ron::from_str::<Margin>(&text).unwrap(), margin);
        let mut scene = crate::core::Scene::new_with_resolution((1000, 500));
        scene.set_margin(margin);
        assert_eq!(
            scene.margin(),
            crate::core::types::Rect::new(-400.0, -225.0, 700.0, 400.0)
        );
        assert!(margin.is_valid((1000, 500)));
        for left in [-1.0, 80.0, f32::NAN, f32::INFINITY] {
            assert!(!Margin { left, ..margin }.is_valid((1000, 500)));
        }
        assert!(
            !Margin {
                bottom: 95.0,
                ..margin
            }
            .is_valid((1000, 500))
        );
        assert!(
            Margin {
                left: 60.0,
                ..margin
            }
            .is_valid((1000, 500))
        );
    }

    #[test]
    fn project_settings_require_exportable_values() {
        assert!(ProjectSettings::is_valid(&ProjectSettings::default()));

        assert!(!ProjectSettings::is_valid(&ProjectSettings {
            resolution: (1279, 720),
            fps: 60,
            margin: Margin::default(),
        }));

        assert!(!ProjectSettings::is_valid(&ProjectSettings {
            resolution: (1280, 720),
            fps: 0,
            margin: Margin::default(),
        }));
    }
}
