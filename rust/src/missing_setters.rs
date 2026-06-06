#[func]
pub fn set_has_quest(&mut self, v: bool) {
    self.has_quest = v;
}

#[func]
pub fn set_quest_kills(&mut self, v: i32) {
    self.quest_kills = v.max(0);
}

#[func]
pub fn set_quest_active(&mut self, v: bool) {
    self.quest_active = v;
}

#[func]
pub fn set_blessings_remaining(&mut self, v: i32) {
    self.blessings_remaining = v.clamp(0, MAX_BLESSINGS);
}

#[func]
pub fn set_has_traded(&mut self, v: bool) {
    self.has_traded = v;
}

#[func]
pub fn set_visit_time_remaining(&mut self, v: f64) {
    self.visit_time_remaining = v.max(0.0);
}

#[func]
pub fn set_markup(&mut self, v: f32) {
    self.markup = v.max(1.0);
}

#[func]
pub fn set_alignment_from_str(&mut self, s: &str) {
    self.alignment = match s {
        "allied" => StudentAlignment::Allied,
        "radicalized" => StudentAlignment::Radicalized,
        _ => StudentAlignment::Neutral,
    };
}

#[func]
pub fn set_boss_exposed(&mut self, v: bool) {
    self.boss_exposed = v;
}
