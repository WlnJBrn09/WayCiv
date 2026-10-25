//! Game length: how many turns a game lasts. Shorter games run faster:
//! research, production, growth, borders, great people, and faith all cost
//! less, in proportion to the length. The rules were balanced for 300
//! turns; a 500-turn game is the standard pace.
//! Infinite games never end on turns and run at 500-turn speed.

use crate::state::Game;

/// The turn count the base costs were balanced for.
const BALANCED_FOR: f32 = 300.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameLength {
    Turns100,
    Turns250,
    #[default]
    Turns500,
    Turns750,
    Turns1000,
    Infinite,
}

impl GameLength {
    pub const ALL: [GameLength; 6] = [
        GameLength::Turns100,
        GameLength::Turns250,
        GameLength::Turns500,
        GameLength::Turns750,
        GameLength::Turns1000,
        GameLength::Infinite,
    ];

    /// The last turn, or `None` for an infinite game.
    pub fn turns(self) -> Option<u32> {
        match self {
            GameLength::Turns100 => Some(100),
            GameLength::Turns250 => Some(250),
            GameLength::Turns500 => Some(500),
            GameLength::Turns750 => Some(750),
            GameLength::Turns1000 => Some(1000),
            GameLength::Infinite => None,
        }
    }

    /// The turns the pace is set by (500 for infinite games).
    pub fn pace_turns(self) -> u32 {
        self.turns().unwrap_or(500)
    }

    /// Cost multiplier: 1/3 for 100 turns, 5/3 for 500, 10/3 for 1000.
    pub fn speed(self) -> f32 {
        self.pace_turns() as f32 / BALANCED_FOR
    }

    pub fn name(self) -> String {
        match self.turns() {
            Some(t) => format!("{t} turns"),
            None => "Infinite".into(),
        }
    }

    /// Parses "100", "250", ..., or "infinite".
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "infinite" | "inf" | "0" => Some(GameLength::Infinite),
            t => GameLength::ALL
                .into_iter()
                .find(|l| l.turns().map(|n| n.to_string()).as_deref() == Some(t)),
        }
    }
}

/// "3000 BC" or "1066 AD".
pub fn year_label(year: i32) -> String {
    if year < 0 {
        format!("{} BC", -year)
    } else {
        format!("{year} AD")
    }
}

impl Game {
    /// Cost multiplier for this game's length.
    pub fn speed(&self) -> f32 {
        self.settings.length.speed()
    }

    /// Scales a base cost by the game's speed.
    pub(crate) fn scaled(&self, cost: i32) -> i32 {
        ((cost as f32 * self.speed()).round() as i32).max(1)
    }

    pub fn turn_limit(&self) -> Option<u32> {
        self.settings.length.turns()
    }

    /// The calendar year of a turn (negative is BC). The calendar runs from
    /// 4000 BC to about 2150 AD on the last turn, stretched to fit the
    /// game's length. It follows the typical pace of research (in all-AI
    /// games the average civilization reaches the Modern era around 1940),
    /// so year-gated governments and policies arrive alongside the techs of
    /// their time. Years pass quickly early on and a year a turn from 1950;
    /// after the last turn (infinite games) it keeps a year a turn.
    pub fn calendar_year(&self, turn: u32) -> i32 {
        let pace = self.settings.length.pace_turns() as f32;
        // The calendar laid out over 500 turns: (turns, years per turn).
        // 1000 BC at turn 75, 1325 at 150, 1700 at 200, 1880 at 250, 1950
        // at 300, 2150 at 500.
        const STEPS: [(f32, f32); 6] = [
            (75.0, 40.0),
            (75.0, 31.0),
            (50.0, 7.5),
            (50.0, 3.6),
            (50.0, 1.4),
            (200.0, 1.0),
        ];
        let standard = |t: f32| -> f32 {
            let mut year = -4000.0;
            let mut left = t;
            for (span, per) in STEPS {
                let step = left.min(span);
                year += step * per;
                left -= step;
                if left <= 0.0 {
                    return year;
                }
            }
            year + left
        };
        let t = (turn.max(1) - 1) as f32;
        if t <= pace {
            standard(t * 500.0 / pace).floor() as i32
        } else {
            standard(500.0).floor() as i32 + (t - pace) as i32
        }
    }

    /// The current calendar year.
    pub fn year(&self) -> i32 {
        self.calendar_year(self.turn)
    }

    /// Whether the calendar has reached `year` (true when there is none).
    pub fn reached(&self, year: Option<i32>) -> bool {
        year.is_none_or(|y| self.year() >= y)
    }

    /// The first turn on which the calendar reaches `year`.
    pub fn turn_of_year(&self, year: i32) -> u32 {
        let mut turn = 1;
        while self.calendar_year(turn) < year && turn < 100_000 {
            turn += 1;
        }
        turn
    }
}

#[cfg(test)]
mod tests {
    use super::GameLength;

    #[test]
    fn shorter_games_are_cheaper() {
        let speeds: Vec<f32> = GameLength::ALL.iter().map(|l| l.speed()).collect();
        assert!(
            speeds[0] < speeds[1]
                && speeds[1] < speeds[2]
                && speeds[2] < speeds[3]
                && speeds[3] < speeds[4]
        );
        assert_eq!(GameLength::Infinite.speed(), GameLength::Turns500.speed());
        assert_eq!(GameLength::Infinite.turns(), None);
        assert_eq!(GameLength::parse("750"), Some(GameLength::Turns750));
        assert_eq!(GameLength::parse("infinite"), Some(GameLength::Infinite));
    }
}
