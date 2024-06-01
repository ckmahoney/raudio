use crate::types::synthesis::*;

pub type Chrom = i8;

/// notepad for Monic Theory

fn fit(a:f32, b:f32) -> f32 {
    if b >= a && b < (a*2.) {
        return b
    } else if b < a {
        return fit(a, b*2.0)
    } else {
        return fit (a, b/2.0)
    }
}

pub fn monae_to_chrom(Monae: Monae, chroma_mod: i8) -> Chrom {
    let (rot, q, monic) = Monae;

    if rot < 0 {
        return monae_to_chrom((0, q, monic), chroma_mod - 7 * rot.abs());
    }

    if rot > 0 {
        return monae_to_chrom((0, q, monic), chroma_mod + 7 * rot.abs());
    }

    let d_mod = chroma_mod % 12;
    let chroma;

    if q == 0 {
        chroma = match monic {
            1 => 0,
            3 => 7,
            5 => 4,
            7 => 10,
            9 => 2,
            _ => {
                println!("Unhandled case for monic {}", 0);
                0
            }
        };
    } else {
        chroma = match monic {
            1 => 0,
            3 => 5,
            5 => 8,
            7 => 2,
            9 => 10,
            _ => {
                println!("Unhandled case for monic {}", 0);
                0
            }
        };
    }

    let mut chroma:Chrom = (chroma + d_mod) % 12;

    while chroma < 0 {
        chroma += 12;
    }

    chroma
}

pub fn tone_to_chrom((register,monae):Tone) -> Chrom {
    const MIDI_OFFSET:i8 = 3; // the register in tone mapped to the minimum applied MIDI value

    let octaveMod = 12 * (register - MIDI_OFFSET);
    octaveMod + monae_to_chrom(monae, 0)
}

pub fn monae_to_freq((rotation, q, monic):&Monae) -> f32 {
    let qq = if *q == 0 {1} else {-1};
    let harmonic_basis = 1.5f32;
    let m = (*monic as f32).powi(qq);
    harmonic_basis.powi(*rotation as i32) * m
}

pub fn tone_to_freq(tone:&Tone) -> f32 {
    let (register, m) = tone;
    fit(2f32.powi(*register as i32), monae_to_freq(m))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::song::happy_birthday;

    #[test]
    fn test_check_notes() {
        let tones:Vec<Tone> = happy_birthday::get_track().parts[0].1[0].iter().map(|p| p.1).collect::<Vec<Tone>>();
        let chroms:Vec<Chrom> = tones.into_iter().map(|t| tone_to_chrom(t)).collect::<Vec<Chrom>>();
            
        let expected:Vec::<i8> = vec![
            43,
            43,
            45,
            43,
            48,
            47,
            43,
            43,
            45,
            43,
            50,
            48,
            43,
            43,
            55,
            52,
            48,
            47,
            45,
            53,
            53,
            52,
            48,
            50,
            48,
        ];
        assert_eq!(expected, chroms);
    }

    #[test]
    fn view_early_monics() {
        let monics:[usize; 7] = [1,3,5, 7, 11, 13, 17 ];
        let os:[f32; 7] = monics.map(|x| fit(1.0, x as f32));
        let us:[f32; 7] = monics.map(|x| fit(1.0, 1f32/x as f32));

        let info = r##"

            # Monic 

                The word "Monic" indicates the prime numbers element witihn the harmonic series. 
                This could also be described as positive primes, usually constrained to be less than 13.
                
                Most of the time we can use 1,3,5. 

                The "space" is either Overtones (O) or Undertones (U).
                Overtones are identical in concept to the harmonics in the harmonic series.
                Undertones are the reciprocal of these values.

                O[1,3,5] = [1,3,5]
                U[1,3,5] = [1, 1/3, 1/5]

            # Basis of Harmony

                For Western music we generally like music that fit well in a piano or guitar. 

                This music is centered around the "Tonic <> Dominant" relationship, also known as {I V I}. A similar effect can be achieved by using a harmonic basis of (3/2) as our value for rotating monics. 

                We apply the exponential variable (r) for rotation. 
                (3/2)^r

                The origin of our tonality is 0. 
                Here we'll apply 0 as r and use it to modulate our monics:

                (3/2)^0 = 1
                O: 1 * [1, 3, 5] = [1, 3, 5]
                U: 1 * [1, 1/3, 1/5] = [1, 1/3, 1/5]
                
                So our O and U series are still [1, 3, 5] and [1, 1/3, 1/5].

                We can describe "neighbor series" by setting r to 1 or -1. Then we get values
                (3/2)^1

                (3/2)^1 = 1.5
                O: 1.5 * [1, 3, 5] = [1.5, 4.5, 7.5]
                U: 1.5 * [1, 1/3, 1/5] = [1.5, 0.5, 0.3]

                (3/2)^-1 = 0.66
                O: 0.66 * [0.666, 2, 2.666]
                U: 0.66 * [1, 1/3, 1/5] = [0.666, 0.222, 0.133]
            
            # Relationship of O an U

                It is satisfying to offer a proof for the conventional Western Major Scale, the Minor Scale, and the fundamental relationship between them.
                To do so we'll need 3 monics, both neighbors, and a sorting function. 
                
                Using O space yields the Major Scale


                O[-1]: 0.66 * [0.666, 2, 2.666]
                O[=0]: 1 * [1, 3, 5] = [1, 3, 5]
                O[+1]: 1.5 * [1, 3, 5] = [1.5, 4.5, 7.5]

                Putting the results all together into one list, from lowest rotation to highest rotation, gives us

                [ 0.666, 2, 2.666, 1, 3, 5, 1.5, 4.5, 7.5]

                Applying our fit function to the results provides


        "##;

        println!("Fit {:?}",(0i32..=110).into_iter().map(|x| (3f32/2f32).powi(x) ).collect::<Vec<f32>>());

        // println!("monics: {:#?}\novertones: {:#?}\nundertones: {:#?}", monics, os, us)
    }

}