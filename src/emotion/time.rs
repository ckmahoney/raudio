use std::collections::HashMap;

let mut emotions = HashMap::new();

emotions.insert("happiness", vec![
    {"attack": "quick, bright", "decay": "short, middle", "sustain": "middle, high", "release": "gradual, smooth"},
    {"attack": "light, rhythmic", "decay": "brief", "sustain": "steady, bright", "release": "gradual, gentle"},
    {"attack": "bright, quick", "decay": "brief", "sustain": "middle, rhythmic", "release": "smooth, gradual"},
    {"attack": "quick, lively", "decay": "short", "sustain": "steady, vibrant", "release": "gradual, gentle"}
]);

emotions.insert("sadness", vec![
    {"attack": "slow, tenuto", "decay": "extended", "sustain": "low, middle", "release": "long, trailing"},
    {"attack": "gentle, lingering", "decay": "extended, flowing", "sustain": "lower intensity", "release": "slow, fading"},
    {"attack": "delicate, prolonged", "decay": "slow, resonant", "sustain": "soft, subdued", "release": "extended, fading"},
    {"attack": "slow, soft", "decay": "extended", "sustain": "subdued, mellow", "release": "trailing, delicate"}
]);

emotions.insert("fear", vec![
    {"attack": "abrupt, sharp", "decay": "variable", "sustain": "low, unsteady", "release": "quick, erratic"},
    {"attack": "sharp, dissonant", "decay": "quick, variable", "sustain": "unsteady, fluctuating", "release": "rapid, erratic"},
    {"attack": "sharp, unsettling", "decay": "erratic, quick", "sustain": "low, quivering", "release": "abrupt, uneven"},
    {"attack": "sharp, abrupt", "decay": "variable", "sustain": "low, unstable", "release": "quick, erratic"}
]);

emotions.insert("disgust", vec![
    {"attack": "harsh, forceful", "decay": "quick", "sustain": "low, muted", "release": "abrupt"},
    {"attack": "harsh, discordant", "decay": "swift", "sustain": "muted, suppressed", "release": "abrupt"},
    {"attack": "discordant, abrupt", "decay": "swift", "sustain": "dissonant, unstable", "release": "quick, disjointed"},
    {"attack": "harsh, forceful", "decay": "rapid", "sustain": "muted, discordant", "release": "abrupt"}
]);

emotions.insert("anger", vec![
    {"attack": "intense, aggressive", "decay": "short", "sustain": "high, relentless", "release": "abrupt, reluctant"},
    {"attack": "strong, explosive", "decay": "short", "sustain": "high, intense", "release": "quick, reluctant"},
    {"attack": "intense, aggressive", "decay": "short", "sustain": "loud, distorted", "release": "sudden, reluctant"},
    {"attack": "intense, forceful", "decay": "brief", "sustain": "high, constant", "release": "abrupt, aggressive"}
]);

emotions.insert("surprise", vec![
    {"attack": "sudden, unexpected", "decay": "quick", "sustain": "variable", "release": "often rapid"},
    {"attack": "sudden, dynamic", "decay": "brief", "sustain": "varied", "release": "often swift"},
    {"attack": "immediate, dynamic", "decay": "brief", "sustain": "varied", "release": "rapid"},
    {"attack": "sudden, unexpected", "decay": "short", "sustain": "unstable", "release": "often quick"}
]);

fn find_unique_values(emotions: &HashMap<&str, Vec<&str>>) -> HashMap<&str, Vec<&str>> {
    let mut unique_values = HashMap::new();

    for profiles in emotions.values() {
        for profile in profiles {
            for (key, value) in profile.iter() {
                unique_values.entry(key).or_insert_with(Vec::new).push(value);
            }
        }
    }

    for values in unique_values.values_mut() {
        values.sort();
        values.dedup();
    }

    unique_values
}