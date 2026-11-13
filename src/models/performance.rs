use crate::models::beatmaps::Beatmap;
use serde::Serialize;

/// What full combos on a beatmap are worth
pub struct BeatmapPerformance {
    pub beatmap: Beatmap,
    pub stars: f32,
    pub pp: PerformancePoints,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PerformancePoints {
    /// The performance of the one accuracy that was asked for
    Single(f32),
    /// The performance of each of the commonly achieved accuracies
    Common(Vec<f32>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_performance_points_without_naming_their_kind() {
        let single = serde_json::to_string(&PerformancePoints::Single(727.5)).unwrap();
        assert_eq!(single, "727.5");

        let common = PerformancePoints::Common(vec![400.0, 380.25, 365.5, 330.0]);
        let common = serde_json::to_string(&common).unwrap();
        assert_eq!(common, "[400.0,380.25,365.5,330.0]");
    }
}
