//! Luzon biome system (Stage 1).
//!
//! The single world is split into 5 large Luzon biomes. A low-frequency biome
//! noise (seeded deterministically from the world seed) is sampled per land cell
//! and bucketed into one of 5 value bands, each band mapping to one biome with a
//! single ground tile. Water is handled by the altitude threshold in `terrain.rs`,
//! not here. Decorations and collision come in a later stage.

/// Tile source id shared by every biome ground tile (world_tileset.png).
pub const BIOME_SOURCE_ID: i32 = 1;

// --- Biome ground tiles: atlas (x, y) on BIOME_SOURCE_ID. Edit to retune. ---
const NCR_TILE: (i32, i32) = (8, 0); // black rock
const CORDILLERA_TILE: (i32, i32) = (0, 0); // dirt with grass
const ILOCOS_TILE: (i32, i32) = (2, 1); // brown sand
const BICOL_TILE: (i32, i32) = (4, 0); // red sand
const CALABARZON_TILE: (i32, i32) = (3, 0); // sand stone

/// Added to the world seed for the biome noise so biome regions do not line up
/// with the altitude noise. Deterministic: same world seed -> same biome layout.
pub const BIOME_SEED_OFFSET: i32 = 7919;

/// Low frequency so biomes form large continuous regions.
pub const BIOME_FREQUENCY: f32 = 0.002;

/// The 5 Luzon biomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Ncr,
    Cordillera,
    Ilocos,
    Bicol,
    Calabarzon,
}

impl Biome {
    /// Atlas (x, y) of this biome's ground tile on [`BIOME_SOURCE_ID`].
    #[inline]
    pub fn land_tile(self) -> (i32, i32) {
        match self {
            Biome::Ncr => NCR_TILE,
            Biome::Cordillera => CORDILLERA_TILE,
            Biome::Ilocos => ILOCOS_TILE,
            Biome::Bicol => BICOL_TILE,
            Biome::Calabarzon => CALABARZON_TILE,
        }
    }
}

/// Upper bounds (on a normalized 0.0..1.0 scale) for each biome's value band.
/// FastNoiseLite clusters values near the middle, so the inner bands are kept
/// narrow and the outer bands wide so every biome gets visible area. The final
/// bound is > 1.0 to catch the top of the range.
const BIOME_BANDS: [(f32, Biome); 5] = [
    (0.30, Biome::Ncr),
    (0.45, Biome::Cordillera),
    (0.55, Biome::Ilocos),
    (0.70, Biome::Bicol),
    (1.01, Biome::Calabarzon),
];

/// Pick a biome from a raw biome-noise sample (FastNoiseLite range ~ -1.0..1.0).
#[inline]
pub fn select_biome(noise: f32) -> Biome {
    // Normalize the noise into 0.0..1.0 before bucketing into bands.
    let v = (noise + 1.0) * 0.5;
    for (upper, biome) in BIOME_BANDS {
        if v < upper {
            return biome;
        }
    }
    Biome::Calabarzon
}
