use noise::{NoiseFn, Perlin};
use noise::Seedable;

// flags    0 == no
//          1 == yes
//          _ == lerp

/// a container for generation options
pub struct Regdata {
    pub raviens: f32,
    pub clifs: f32,
    pub fiords: f32,
}

/// generate regdata using seed with perlin noise
pub fn newreg(seed: (f32,f32)) -> Regdata {
    let raviens = Perlin::new();
    raviens.set_seed(100);
    let raviens = raviens.get([seed.0 as f64/70.0,seed.1 as f64/70.0]) as f32-0.5;
    let raviens = (raviens.max(0.0)*10.0).min(1.0);
    
    let clifs = Perlin::new();
    clifs.set_seed(200);
    let clifs = clifs.get([seed.0 as f64/700.0,seed.1 as f64/700.0]) as f32-0.5;
    let clifs = (clifs.max(0.0)*10.0).min(1.0);
    
    let fiords = Perlin::new();
    fiords.set_seed(300);
    let fiords = fiords.get([seed.0 as f64/700.0,seed.1 as f64/700.0]) as f32-0.5;
    let fiords = (fiords.max(0.0)*10.0).min(1.0);
    
    Regdata {
        raviens,
        clifs,
        fiords
    }
}
