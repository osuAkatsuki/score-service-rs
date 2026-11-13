use axum::response::{IntoResponse, Response};

/// What the osu! client is told about a beatmap when it asks to rate it
#[derive(Debug, PartialEq)]
pub enum BeatmapRatingView {
    /// The user has yet to rate the beatmap
    Rateable,
    /// Only beatmaps with a leaderboard can be rated
    Unranked,
    /// The user has rated the beatmap before
    AlreadyRated { rating: f32 },
    /// The rating of the user was added to the ratings of the beatmap
    Rated { rating: f32 },
}

impl BeatmapRatingView {
    pub fn osu_format(&self) -> String {
        match self {
            Self::Rateable => "ok".to_owned(),
            Self::Unranked => "not ranked".to_owned(),
            Self::AlreadyRated { rating } => format!("alreadyvoted\n{rating:.2}"),
            Self::Rated { rating } => format!("alreadyvoting\n{rating:.2}"),
        }
    }
}

impl IntoResponse for BeatmapRatingView {
    fn into_response(self) -> Response {
        self.osu_format().into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_the_client_whether_the_beatmap_can_be_rated() {
        assert_eq!(BeatmapRatingView::Rateable.osu_format(), "ok");
        assert_eq!(BeatmapRatingView::Unranked.osu_format(), "not ranked");
    }

    #[test]
    fn sends_the_rating_of_the_beatmap_on_its_own_line() {
        let already_rated = BeatmapRatingView::AlreadyRated { rating: 10.0 };
        assert_eq!(already_rated.osu_format(), "alreadyvoted\n10.00");

        let rated = BeatmapRatingView::Rated { rating: 8.666667 };
        assert_eq!(rated.osu_format(), "alreadyvoting\n8.67");
    }
}
