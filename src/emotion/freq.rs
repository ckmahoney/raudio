use std::collections::HashMap;

// Define enums for Timbre Descriptor ranges
#[derive(Debug, Clone)]
enum IntensityLevel {
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

// Define enums for Dynamic Spectral Modifiers
#[derive(Debug, Clone)]
enum Spread {
    Narrow,
    Moderate,
    Wide,
}

#[derive(Debug, Clone)]
enum Evolution {
    Static,
    SlowEvolution,
    RapidEvolution,
    CyclicEvolution,
}

#[derive(Debug, Clone)]
enum Balance {
    BassHeavy,
    MidFocused,
    TrebleHeavy,
    Balanced,
}

#[derive(Debug, Clone)]
enum Complexity {
    Simple,
    Moderate,
    Complex,
}

// Define Timbre Descriptor Struct
#[derive(Debug, Clone)]
struct TimbreDescriptor {
    brightness: Option<IntensityLevel>,
    warmth: Option<IntensityLevel>,
    metallicity: Option<IntensityLevel>,
    mellowness: Option<IntensityLevel>,
    resonance: Option<IntensityLevel>,
    clarity: Option<IntensityLevel>,
    roughness: Option<IntensityLevel>,
    sharpness: Option<IntensityLevel>,
    depth: Option<IntensityLevel>,
    airiness: Option<IntensityLevel>,
}

// Define Spectral Modifier Struct
#[derive(Debug, Clone)]
struct SpectralMod {
    spread: Spread,
    evolution: Evolution,
    balance: Balance,
    complexity: Complexity,
}

// Define Emotion Struct
#[derive(Debug, Clone)]
struct Emotion {
    timbre_descriptors: TimbreDescriptor,
    spectral_mods: SpectralMod,
}


fn main() {
    // Initialize a HashMap for emotions with multiple representations
    let mut emotions: HashMap<&str, Vec<Emotion>> = HashMap::new();

    // Adding Happiness Variants
    emotions.insert("happiness", vec![
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level4),
                airiness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Moderate,
            },
        },
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level5),
                airiness: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Moderate,
            },
        },
        // Euphoric Happiness
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level5),
                airiness: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Complex,
            },
        },
        // Content Happiness
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                warmth: Some(IntensityLevel::Level3),
                mellowness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::Balanced,
                complexity: Complexity::Simple,
            },
        },
        // Exuberant Joy
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level5),
                airiness: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Complex,
            },
        },
        // Serene Joy
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                warmth: Some(IntensityLevel::Level3),
                mellowness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::Balanced,
                complexity: Complexity::Simple,
            },
        },
    ]);

    // Adding Sadness Variants
    emotions.insert("sadness", vec![
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                warmth: Some(IntensityLevel::Level4),
                mellowness: Some(IntensityLevel::Level4),
                depth: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::SlowEvolution,
                balance: Balance::BassHeavy,
                complexity: Complexity::Simple,
            },
        },
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                warmth: Some(IntensityLevel::Level5),
                mellowness: Some(IntensityLevel::Level5),
                depth: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::SlowEvolution,
                balance: Balance::BassHeavy,
                complexity: Complexity::Simple,
            },
        },
        // Melancholic Sadness
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                mellowness: Some(IntensityLevel::Level4),
                depth: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::SlowEvolution,
                balance: Balance::BassHeavy,
                complexity: Complexity::Simple,
            },
        },
        // Despair
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level2),
                depth: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::Static,
                balance: Balance::BassHeavy,
                complexity: Complexity::Moderate,
            },
        },
        // Grieving
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                depth: Some(IntensityLevel::Level5),
                roughness: Some(IntensityLevel::Level2),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::Static,
                balance: Balance::BassHeavy,
                complexity: Complexity::Simple,
            },
        },
        // Nostalgic
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                mellowness: Some(IntensityLevel::Level4),
                warmth: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Moderate,
            },
        },
    ]);

    // Adding Anger Variants
    emotions.insert("anger", vec![
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level4),
                sharpness: Some(IntensityLevel::Level4),
                metallicity: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::CyclicEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Complex,
            },
        },
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level5),
                sharpness: Some(IntensityLevel::Level5),
                metallicity: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::CyclicEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Complex,
            },
        },
        // Frustration
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level3),
                sharpness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::CyclicEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Moderate,
            },
        },
        // Rage
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level5),
                sharpness: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Complex,
            },
        },
        // Annoyance
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level3),
                metallicity: Some(IntensityLevel::Level2),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::CyclicEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Simple,
            },
        },
        // Hostility
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level5),
                sharpness: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Complex,
            },
        },
    ]);

    // Adding Fear Variants
    emotions.insert("fear", vec![
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                sharpness: Some(IntensityLevel::Level3),
                clarity: Some(IntensityLevel::Level2),
                airiness: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Moderate,
            },
        },
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                sharpness: Some(IntensityLevel::Level4),
                clarity: Some(IntensityLevel::Level3),
                airiness: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Complex,
            },
        },
        // Anxiety
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                sharpness: Some(IntensityLevel::Level3),
                clarity: Some(IntensityLevel::Level2),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Moderate,
            },
        },
        // Terror
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                sharpness: Some(IntensityLevel::Level5),
                airiness: Some(IntensityLevel::Level5),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Complex,
            },
        },
        // Worry
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                clarity: Some(IntensityLevel::Level2),
                sharpness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Moderate,
            },
        },
        // Dread
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                sharpness: Some(IntensityLevel::Level5),
                roughness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::Static,
                balance: Balance::BassHeavy,
                complexity: Complexity::Complex,
            },
        },
    ]);

    // Adding Disgust Variants
    emotions.insert("disgust", vec![
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level3),
                depth: Some(IntensityLevel::Level2),
                clarity: Some(IntensityLevel::Level1),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::Static,
                balance: Balance::BassHeavy,
                complexity: Complexity::Simple,
            },
        },
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level4),
                depth: Some(IntensityLevel::Level3),
                clarity: Some(IntensityLevel::Level2),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::Static,
                balance: Balance::MidFocused,
                complexity: Complexity::Moderate,
            },
        },
        // Disapproval
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level2),
                depth: Some(IntensityLevel::Level2),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::Static,
                balance: Balance::MidFocused,
                complexity: Complexity::Simple,
            },
        },
        // Revulsion
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level4),
                depth: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::BassHeavy,
                complexity: Complexity::Moderate,
            },
        },
        // Dislike
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level2),
                depth: Some(IntensityLevel::Level2),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Narrow,
                evolution: Evolution::Static,
                balance: Balance::MidFocused,
                complexity: Complexity::Simple,
            },
        },
        // Abhorrence
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                roughness: Some(IntensityLevel::Level5),
                metallicity: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::SlowEvolution,
                balance: Balance::BassHeavy,
                complexity: Complexity::Moderate,
            },
        },
    ]);

    // Adding Surprise Variants
    emotions.insert("surprise", vec![
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level4),
                airiness: Some(IntensityLevel::Level3),
                sharpness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::Balanced,
                complexity: Complexity::Moderate,
            },
        },
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level5),
                airiness: Some(IntensityLevel::Level5),
                sharpness: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Complex,
            },
        },
        // Pleasant Surprise
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level4),
                airiness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::RapidEvolution,
                balance: Balance::Balanced,
                complexity: Complexity::Moderate,
            },
        },
        // Shock
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level5),
                sharpness: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Complex,
            },
        },
        // Astonishment
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level5),
                airiness: Some(IntensityLevel::Level4),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::Balanced,
                complexity: Complexity::Complex,
            },
        },
        // Bewilderment
        Emotion {
            timbre_descriptors: TimbreDescriptor {
                clarity: Some(IntensityLevel::Level3),
                sharpness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Moderate,
                evolution: Evolution::RapidEvolution,
                balance: Balance::MidFocused,
                complexity: Complexity::Moderate,
            },
        },
    ]);

    // Accessing and Printing the emotions data
    for (emotion, variants) in &emotions {
        println!("{}:", emotion.to_uppercase());
        for (i, variant) in variants.iter().enumerate() {
            println!("  Variant {}: {:?}", i + 1, variant);
        }
        println!();
    }
}

// Helper function to convert IntensityLevel to String
fn intensity_to_string(intensity: Option<&IntensityLevel>) -> String {
    match intensity {
        Some(level) => match level {
            IntensityLevel::Level1 => "Level 1".to_string(),
            IntensityLevel::Level2 => "Level 2".to_string(),
            IntensityLevel::Level3 => "Level 3".to_string(),
            IntensityLevel::Level4 => "Level 4".to_string(),
            IntensityLevel::Level5 => "Level 5".to_string(),
        },
        None => "Not specified".to_string(),
    }
}

// Helper function to describe an emotion variant
fn describe_variant(emotion: &str, variant: &Emotion) -> String {
    let timbre = &variant.timbre_descriptors;
    let mods = &variant.spectral_mods;

    format!(
        "{} variant: This expression is characterized by a timbre with brightness {}, warmth {}, metallicity {}, mellowness {}, resonance {}, clarity {}, roughness {}, sharpness {}, depth {}, and airiness {}. The spectral characteristics include a spread of {:?}, an evolution of {:?}, a balance of {:?}, and a complexity of {:?}.",
        emotion.to_uppercase(),
        intensity_to_string(timbre.brightness.as_ref()),
        intensity_to_string(timbre.warmth.as_ref()),
        intensity_to_string(timbre.metallicity.as_ref()),
        intensity_to_string(timbre.mellowness.as_ref()),
        intensity_to_string(timbre.resonance.as_ref()),
        intensity_to_string(timbre.clarity.as_ref()),
        intensity_to_string(timbre.roughness.as_ref()),
        intensity_to_string(timbre.sharpness.as_ref()),
        intensity_to_string(timbre.depth.as_ref()),
        intensity_to_string(timbre.airiness.as_ref()),
        mods.spread, mods.evolution, mods.balance, mods.complexity
    )
}

// Function to print a summary of all emotions and their variants
fn print_module_summary(emotions: &HashMap<&str, Vec<Emotion>>) {
    for (emotion, variants) in emotions {
        println!("{}:", emotion.to_uppercase());
        for (i, variant) in variants.iter().enumerate() {
            let description = describe_variant(emotion, variant);
            println!("  Variant {}: {}", i + 1, description);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_variant() {
        // Create a sample emotion variant
        let sample_emotion = Emotion {
            timbre_descriptors: TimbreDescriptor {
                brightness: Some(IntensityLevel::Level4),
                airiness: Some(IntensityLevel::Level3),
                ..Default::default()
            },
            spectral_mods: SpectralMod {
                spread: Spread::Wide,
                evolution: Evolution::RapidEvolution,
                balance: Balance::TrebleHeavy,
                complexity: Complexity::Moderate,
            },
        };

        // Call describe_variant function
        let description = describe_variant("Happiness", &sample_emotion);

        // Check if the description contains certain expected strings
        assert!(description.contains("HAPPINESS variant:"));
        assert!(description.contains("brightness Level 4"));
        assert!(description.contains("airiness Level 3"));
        assert!(description.contains("spread of Wide"));
        assert!(description.contains("evolution of RapidEvolution"));
        assert!(description.contains("balance of TrebleHeavy"));
        assert!(description.contains("complexity of Moderate"));
    }

    #[test]
    fn test_print_module_summary() {
        // Create a sample emotions HashMap
        let mut emotions: HashMap<&str, Vec<Emotion>> = HashMap::new();
        emotions.insert("test", vec![
            Emotion {
                timbre_descriptors: TimbreDescriptor::default(),
                spectral_mods: SpectralMod {
                    spread: Spread::Narrow,
                    evolution: Evolution::Static,
                    balance: Balance::Balanced,
                    complexity: Complexity::Simple,
                },
            },
        ]);

        // Since print_module_summary prints to the console, 
        // we can't capture its output in a standard way.
        // Instead, we'll just call it to ensure it doesn't panic.
        print_module_summary(&emotions);
    }
}

