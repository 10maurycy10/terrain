use noise::{NoiseFn, Perlin};
use noise::Seedable;

pub struct Regdata {
    pub raviens: bool,
    pub clifs: bool,
    pub fiords: bool,
}


pub fn newreg(seed: (f32,f32)) -> Regdata {
    let raviens = Perlin::new();
    raviens.set_seed(100);
    let raviens = raviens.get([seed.0 as f64/70.0,seed.1 as f64/70.0])>0.5;
    
    let clifs = Perlin::new();
    clifs.set_seed(200);
    let clifs = clifs.get([seed.0 as f64/700.0,seed.1 as f64/700.0])>0.5;
    
    let fiords = Perlin::new();
    fiords.set_seed(300);
    let fiords = fiords.get([seed.0 as f64/700.0,seed.1 as f64/700.0])>0.5;
    
    Regdata {
        raviens,
        clifs,
        fiords
    }
}
