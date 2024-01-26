use itertools::Either;

type Q = Either<i8, i8>;

pub struct Phrase {
    pub cpc: i8,
    pub root: f32,
    pub q: Q,
}

pub struct Globe {
    pub dur: f32,
    pub origin: f32,
    pub q: Q,
    pub cps: f32,
}
type Range = f32;

pub type PhraseMod = Box<dyn Fn(f32) -> Range>;

// for all harmonic ratios of time, 
// create a linear ramp at that scale
pub fn all(globe:&Globe, phrase:&Phrase) -> Vec<PhraseMod> {
    let n:i8 = 12;
    
    let mut oscs = Vec::<PhraseMod>::with_capacity(n as usize);
    
    for i in 0..n-1 {
        let length = phrase.cpc as f32 * 2f32.powf(i as f32);
        let osc: PhraseMod = Box::new(move |cycle: f32| -> Range {
            cycle.rem_euclid(length) / length
        });

        oscs.push(Box::new(osc))
    }

    oscs
}

#[test]
fn test_run() {
    let g:Globe = Globe {
        dur: 128.0,
        origin: 1.05,
        q: Either::Left(0),
        cps: 1.2
    };
    let p:Phrase = Phrase {
        cpc: 1,
        root: 1.05,
        q: Either::Left(0)
    };

    let oscs = all(&g, &p);
    for osc in oscs {
        println!("Got this value for the oscirrator {}", osc(2.0f32.powf(4.)));
    }
}