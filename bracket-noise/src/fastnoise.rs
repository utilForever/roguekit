// A compatibility wrapper for bracket-noise's FastNoise API.
// Coherent noise generation is delegated to FastNoiseLite where possible.

use fastnoise_lite as fnl;
use fastnoise_lite::FastNoiseLite;
use std::num::Wrapping;

#[derive(Debug, PartialEq, Copy, Clone)]
/// Type of noise to generate
pub enum NoiseType {
    Value,
    ValueFractal,
    Perlin,
    PerlinFractal,
    Simplex,
    SimplexFractal,
    Cellular,
    WhiteNoise,
    Cubic,
    CubicFractal,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Interpolation function to use
pub enum Interp {
    Linear,
    Hermite,
    Quintic,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Fractal function to use
pub enum FractalType {
    FBM,
    Billow,
    RigidMulti,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Cellular noise distance function to use
pub enum CellularDistanceFunction {
    Euclidean,
    Manhattan,
    Natural,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// What type of cellular noise result do you want
pub enum CellularReturnType {
    CellValue,
    Distance,
    Distance2,
    Distance2Add,
    Distance2Sub,
    Distance2Mul,
    Distance2Div,
}

pub struct FastNoise {
    inner: FastNoiseLite,
    seed: u64,
    frequency: f32,
    interp: Interp,
    noise_type: NoiseType,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    fractal_type: FractalType,
    cellular_distance_function: CellularDistanceFunction,
    cellular_return_type: CellularReturnType,
    cellular_distance_index: (i32, i32),
    cellular_jitter: f32,
    gradient_perturb_amp: f32,
}

const FN_CELLULAR_INDEX_MAX: i32 = 3;

const X_PRIME: i32 = 1619;
const Y_PRIME: i32 = 31337;
const Z_PRIME: i32 = 6971;
const W_PRIME: i32 = 1013;

impl Default for FastNoise {
    fn default() -> Self {
        Self::new()
    }
}

impl FastNoise {
    /// Creates a new noise instance, using simplex noise defaults.
    pub fn new() -> FastNoise {
        Self::seeded(1337)
    }

    /// Creates a new noise instance, using simplex noise defaults and specifying a random seed.
    pub fn seeded(seed: u64) -> FastNoise {
        let mut noise = FastNoise {
            inner: FastNoiseLite::with_seed(to_fnl_seed(seed)),
            seed,
            frequency: 0.0,
            interp: Interp::Quintic,
            noise_type: NoiseType::Simplex,
            octaves: 3,
            lacunarity: 2.0,
            gain: 0.5,
            fractal_type: FractalType::FBM,
            cellular_distance_function: CellularDistanceFunction::Euclidean,
            cellular_return_type: CellularReturnType::CellValue,
            cellular_distance_index: (0, 1),
            cellular_jitter: 0.45,
            gradient_perturb_amp: 1.0,
        };

        noise.sync_inner_settings();
        noise
    }

    fn sync_inner_settings(&mut self) {
        self.inner.set_seed(Some(to_fnl_seed(self.seed)));
        self.inner.set_frequency(Some(self.frequency));
        self.inner
            .set_noise_type(Some(map_noise_type(self.noise_type)));

        let fractal_type = if is_fractal_noise_type(self.noise_type) {
            map_fractal_type(self.fractal_type)
        } else {
            fnl::FractalType::None
        };

        self.inner.set_fractal_type(Some(fractal_type));
        self.inner.set_fractal_octaves(Some(self.octaves.max(1)));
        self.inner.set_fractal_lacunarity(Some(self.lacunarity));
        self.inner.set_fractal_gain(Some(self.gain));

        self.inner
            .set_cellular_distance_function(Some(map_cellular_distance_function(
                self.cellular_distance_function,
            )));
        self.inner
            .set_cellular_return_type(Some(map_cellular_return_type(self.cellular_return_type)));
        self.inner.set_cellular_jitter(Some(self.cellular_jitter));
    }

    /// Re-seeds the noise system with a new seed.
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        self.sync_inner_settings();
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.sync_inner_settings();
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_interp(&mut self, interp: Interp) {
        self.interp = interp;
    }

    pub fn get_interp(&self) -> Interp {
        self.interp
    }

    pub fn set_noise_type(&mut self, nt: NoiseType) {
        self.noise_type = nt;
        self.sync_inner_settings();
    }

    pub fn get_noise_type(&self) -> NoiseType {
        self.noise_type
    }

    pub fn set_fractal_octaves(&mut self, octaves: i32) {
        self.octaves = octaves;
        self.sync_inner_settings();
    }

    pub fn get_fractal_octaves(&self) -> i32 {
        self.octaves
    }

    pub fn set_fractal_lacunarity(&mut self, lacunarity: f32) {
        self.lacunarity = lacunarity;
        self.sync_inner_settings();
    }

    pub fn get_fractal_lacunarity(&self) -> f32 {
        self.lacunarity
    }

    pub fn set_fractal_gain(&mut self, gain: f32) {
        self.gain = gain;
        self.sync_inner_settings();
    }

    pub fn get_fractal_gain(&self) -> f32 {
        self.gain
    }

    pub fn set_fractal_type(&mut self, fractal_type: FractalType) {
        self.fractal_type = fractal_type;
        self.sync_inner_settings();
    }

    pub fn get_fractal_type(&self) -> FractalType {
        self.fractal_type
    }

    pub fn set_cellular_distance_function(
        &mut self,
        cellular_distance_function: CellularDistanceFunction,
    ) {
        self.cellular_distance_function = cellular_distance_function;
        self.sync_inner_settings();
    }

    pub fn get_cellular_distance_function(&self) -> CellularDistanceFunction {
        self.cellular_distance_function
    }

    pub fn set_cellular_return_type(&mut self, cellular_return_type: CellularReturnType) {
        self.cellular_return_type = cellular_return_type;
        self.sync_inner_settings();
    }

    pub fn get_cellular_return_type(&self) -> CellularReturnType {
        self.cellular_return_type
    }

    pub fn get_cellular_distance_indices(&self) -> (i32, i32) {
        self.cellular_distance_index
    }

    pub fn set_cellular_distance_indices(&mut self, i1: i32, i2: i32) {
        let min = i32::min(i1, i2).clamp(0, FN_CELLULAR_INDEX_MAX);
        let max = i32::max(i1, i2).clamp(0, FN_CELLULAR_INDEX_MAX);

        self.cellular_distance_index = (min, max);
    }

    pub fn set_cellular_jitter(&mut self, jitter: f32) {
        self.cellular_jitter = jitter;
        self.sync_inner_settings();
    }

    pub fn get_cellular_jitter(&self) -> f32 {
        self.cellular_jitter
    }

    pub fn set_gradient_perterb_amp(&mut self, gradient_perturb_amp: f32) {
        self.gradient_perturb_amp = gradient_perturb_amp;
    }

    pub fn get_gradient_perterb_amp(&self) -> f32 {
        self.gradient_perturb_amp
    }

    pub fn get_noise(&self, x: f32, y: f32) -> f32 {
        match self.noise_type {
            NoiseType::WhiteNoise => self.get_white_noise(x, y),
            _ => self.inner.get_noise_2d(x, y),
        }
    }

    pub fn get_noise3d(&self, x: f32, y: f32, z: f32) -> f32 {
        match self.noise_type {
            NoiseType::WhiteNoise => self.get_white_noise3d(x, y, z),
            _ => self.inner.get_noise_3d(x, y, z),
        }
    }

    pub fn index2d_12(&self, offset: u8, x: i32, y: i32) -> u8 {
        self.hash_2d(offset, x, y) % 12
    }

    pub fn index3d_12(&self, offset: u8, x: i32, y: i32, z: i32) -> u8 {
        self.hash_3d(offset, x, y, z) % 12
    }

    pub fn index4d_32(&self, offset: u8, x: i32, y: i32, z: i32, w: i32) -> u8 {
        self.hash_4d(offset, x, y, z, w) & 31
    }

    pub fn index2d_256(&self, offset: u8, x: i32, y: i32) -> u8 {
        self.hash_2d(offset, x, y)
    }

    pub fn index3d_256(&self, offset: u8, x: i32, y: i32, z: i32) -> u8 {
        self.hash_3d(offset, x, y, z)
    }

    pub fn index4d_256(&self, offset: u8, x: i32, y: i32, z: i32, w: i32) -> u8 {
        self.hash_4d(offset, x, y, z, w)
    }

    fn get_white_noise(&self, x: f32, y: f32) -> f32 {
        let xc = x.to_bits() as i32;
        let yc = y.to_bits() as i32;

        val_coord_2d(to_fnl_seed(self.seed), xc ^ (xc >> 16), yc ^ (yc >> 16))
    }

    fn get_white_noise3d(&self, x: f32, y: f32, z: f32) -> f32 {
        let xc = x.to_bits() as i32;
        let yc = y.to_bits() as i32;
        let zc = z.to_bits() as i32;

        val_coord_3d(
            to_fnl_seed(self.seed),
            xc ^ (xc >> 16),
            yc ^ (yc >> 16),
            zc ^ (zc >> 16),
        )
    }

    fn hash_2d(&self, offset: u8, x: i32, y: i32) -> u8 {
        let seed = to_fnl_seed(self.seed).wrapping_add(offset as i32);
        let value = val_coord_2d(seed, x, y);

        ((value.abs() * 255.0) as u8).wrapping_add(offset)
    }

    fn hash_3d(&self, offset: u8, x: i32, y: i32, z: i32) -> u8 {
        let seed = to_fnl_seed(self.seed).wrapping_add(offset as i32);
        let value = val_coord_3d(seed, x, y, z);

        ((value.abs() * 255.0) as u8).wrapping_add(offset)
    }

    fn hash_4d(&self, offset: u8, x: i32, y: i32, z: i32, w: i32) -> u8 {
        let seed = to_fnl_seed(self.seed).wrapping_add(offset as i32);
        let value = val_coord_4d(seed, x, y, z, w);

        ((value.abs() * 255.0) as u8).wrapping_add(offset)
    }
}

fn to_fnl_seed(seed: u64) -> i32 {
    seed as i32
}

fn map_noise_type(noise_type: NoiseType) -> fnl::NoiseType {
    match noise_type {
        NoiseType::Value | NoiseType::ValueFractal => fnl::NoiseType::Value,
        NoiseType::Perlin | NoiseType::PerlinFractal => fnl::NoiseType::Perlin,

        // The legacy Simplex implementation is mapped to FastNoiseLite's closest equivalent.
        NoiseType::Simplex | NoiseType::SimplexFractal => fnl::NoiseType::OpenSimplex2,

        NoiseType::Cellular => fnl::NoiseType::Cellular,
        NoiseType::Cubic | NoiseType::CubicFractal => fnl::NoiseType::ValueCubic,
        NoiseType::WhiteNoise => fnl::NoiseType::Value,
    }
}

fn is_fractal_noise_type(noise_type: NoiseType) -> bool {
    matches!(
        noise_type,
        NoiseType::ValueFractal
            | NoiseType::PerlinFractal
            | NoiseType::SimplexFractal
            | NoiseType::CubicFractal
    )
}

fn map_fractal_type(fractal_type: FractalType) -> fnl::FractalType {
    match fractal_type {
        FractalType::FBM => fnl::FractalType::FBm,
        FractalType::Billow => fnl::FractalType::PingPong,
        FractalType::RigidMulti => fnl::FractalType::Ridged,
    }
}

fn map_cellular_distance_function(
    function: CellularDistanceFunction,
) -> fnl::CellularDistanceFunction {
    match function {
        // The legacy Euclidean path used squared-distance behavior.
        CellularDistanceFunction::Euclidean => fnl::CellularDistanceFunction::EuclideanSq,
        CellularDistanceFunction::Manhattan => fnl::CellularDistanceFunction::Manhattan,
        CellularDistanceFunction::Natural => fnl::CellularDistanceFunction::Hybrid,
    }
}

fn map_cellular_return_type(return_type: CellularReturnType) -> fnl::CellularReturnType {
    match return_type {
        CellularReturnType::CellValue => fnl::CellularReturnType::CellValue,
        CellularReturnType::Distance => fnl::CellularReturnType::Distance,
        CellularReturnType::Distance2 => fnl::CellularReturnType::Distance2,
        CellularReturnType::Distance2Add => fnl::CellularReturnType::Distance2Add,
        CellularReturnType::Distance2Sub => fnl::CellularReturnType::Distance2Sub,
        CellularReturnType::Distance2Mul => fnl::CellularReturnType::Distance2Mul,
        CellularReturnType::Distance2Div => fnl::CellularReturnType::Distance2Div,
    }
}

fn val_coord_2d(seed: i32, x: i32, y: i32) -> f32 {
    let mut n = Wrapping(seed);
    n ^= Wrapping(X_PRIME) * Wrapping(x);
    n ^= Wrapping(Y_PRIME) * Wrapping(y);

    (n * n * n * Wrapping(60493)).0 as f32 / 2_147_483_648.0
}

fn val_coord_3d(seed: i32, x: i32, y: i32, z: i32) -> f32 {
    let mut n = Wrapping(seed);
    n ^= Wrapping(X_PRIME) * Wrapping(x);
    n ^= Wrapping(Y_PRIME) * Wrapping(y);
    n ^= Wrapping(Z_PRIME) * Wrapping(z);

    (n * n * n * Wrapping(60493)).0 as f32 / 2_147_483_648.0
}

fn val_coord_4d(seed: i32, x: i32, y: i32, z: i32, w: i32) -> f32 {
    let mut n = Wrapping(seed);
    n ^= Wrapping(X_PRIME) * Wrapping(x);
    n ^= Wrapping(Y_PRIME) * Wrapping(y);
    n ^= Wrapping(Z_PRIME) * Wrapping(z);
    n ^= Wrapping(W_PRIME) * Wrapping(w);

    (n * n * n * Wrapping(60493)).0 as f32 / 2_147_483_648.0
}
