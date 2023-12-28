pub struct WorkoutCalendar {
    pub name: String,
    pub weeks: Vec<WeekTemplate>,
    pub scaling: ScalingSchedule,
}

pub struct WeekTemplate {
    pub days: Vec<DayTemplate>,
}

pub struct DayTemplate {
    pub workouts: Vec<Workout>,
}

pub struct ScalingSchedule {
    pub starting_load: u16,
    pub weekly_scaling: Vec<ScaleEntry>,
    pub breaks: Vec<TrainingBreak>,
}

pub enum ScaleEntry {
    Relative(RelativeLoad),
    Absolute(AbsoluteLoad),
}

pub struct RelativeLoad {
    pub change: i16,
    pub relative_to: i16,
}

pub struct AbsoluteLoad {
    pub load: i16,
}

pub struct TrainingBreak {
    pub start_week: u16,
    pub start_day: u8,
    pub end_week: u16,
    pub end_day: u8,
}

pub struct Workout {
    pub name: String,
    pub sessions: Vec<WorkoutSession>,
}

pub enum WorkoutSession {
    Duration(DurationSession),
    Distance(DistanceSession),
    Resistance(ResistanceSession),
    Group(GroupSession),
}

pub struct GroupSession {
    pub sessions: Vec<WorkoutSession>,
    pub repetitions: u8,
}

pub struct DurationSession {
    pub duration: u32,
    pub zones: Optional<Vec<u8>>,
    pub pace_range: Optional<(PacePerKm, PacePerKm)>,
    pub power_range: Optional<(Power, Power)>,
}

pub struct DistanceSession {
    pub distance_m: u32,
    pub zones: Optional<Vec<u8>>,
    pub pace_range: Optional<(PacePerKm, PacePerKm)>,
    pub power_range: Optional<(Power, Power)>,
}

pub struct PacePerKm(u16);

pub struct Power(u16);

pub struct ResistanceSession {
    pub exercise: String,
    pub reps: u8,
    pub sets: u8,
    pub weight_g: u32,
    pub rest_s: u32,
}
