use render::engrave;

/// Methods for examining a preset from any desired angle
use super::*;
use crate::analysis::melody::find_reach;
use crate::presets::PresetPack;



/// iterations happen from first to last. 
/// so sort these in an order that matches which stems you want to read first.


const VISIBILTYS:[Visibility; 4] = [
    Visibility::Visible,
    Visibility::Background,
    Visibility::Foreground,
    Visibility::Hidden,
];

const ENERGYS:[Energy; 3] = [
    Energy::Medium,
    Energy::Low,
    Energy::High,
];

const PRESENCES:[Presence; 3] = [
    Presence::Staccatto,
    Presence::Legato,
    Presence::Tenuto,
];

// pub struct Coverage<'hyper> {
//     label: &'hyper str,
//     mode: Vec<Mode>,
//     role: Vec<Role>,
//     register: Vec<Register>,
//     visibility: Vec<Visibility>,
//     energy: Vec<&'hyper Energy>,
//     presence: Vec<&'hyper Presence>
// }

pub type LabelledArf = (String, Arf);

/// Given a melody, role, and mode, 
/// Create all variations possible (with respect to VEP parameters)
pub fn iter_all_vep<'render>(label: &'render str, role: Role, mode: Mode, melody: &'render &Melody<Note>) -> Vec<LabelledArf> {
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(melody);
    let n_coverages = VISIBILTYS.len() * ENERGYS.len() * PRESENCES.len();
    let mut sources:Vec<LabelledArf> = Vec::with_capacity(n_coverages);
    let mut i = 0;

    for &visibility in &VISIBILTYS {
        for &energy in &ENERGYS {
            for &presence in &PRESENCES {
                let sample_name = format!("{}_v={}_e={}_p={}_index={}", label, visibility, energy, presence, i);
                sources.push((sample_name, Arf {
                    mode,
                    role,
                    register: lowest_register,
                    visibility,
                    energy,
                    presence
                }));
                i += 1;
            }
        }
    };
    
    sources
}

/// Given a melody, Labelled Arfs, and a preset to splay, 
/// Render each labelled arf using the preset into destination_dir.
pub fn render_labelled_arf<Pack: PresetPack>(
    destination_dir: &str,
    root:f32,
    cps: f32,
    melody:&Melody<Note>,
    (label, arf): &LabelledArf,
    pack: Pack,
    ) {
        let group_reverbs:Vec<ReverbParams> = vec![];
        let keep_stems = Some(destination_dir);
        let stems:Vec<Stem> = vec![
            Pack::to_stem(cps, melody, arf)
        ];
        let samples = render::combiner_with_reso(cps, root, stems, &group_reverbs, keep_stems);
        let filename = format!("{}/{}.wav", destination_dir, label);
        engrave::samples(SR, &samples, &filename);
}
