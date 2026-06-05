//! Luzon biome system (Stages 1-2).
//!
//! Stage 1: the single world is split into 5 large Luzon biomes. A low-frequency
//! biome noise (seeded deterministically from the world seed) is sampled per land
//! cell and bucketed into one of 5 value bands, one band per biome.
//!
//! Stage 2: each biome owns 2-3 same-family ground-tile variants. After the biome
//! is picked, a medium-frequency variant noise is sampled at the same cell so
//! nearby cells tend to share a variant, forming small patchy clusters rather than
//! per-cell scatter. Water is handled by the altitude threshold in `terrain.rs`,
//! not here. Decorations and collision come in a later stage.

/// Tile source id shared by every biome ground tile (world_tileset.png).
pub const BIOME_SOURCE_ID: i32 = 1;

// --- Biome ground-tile variants: atlas (x, y) on BIOME_SOURCE_ID, same color
// family per biome. First entry doubles as the Stage 1 base tile. Edit to retune. ---
const NCR_TILES: [(i32, i32); 2] = [(8, 0), (8, 1)]; // black rock
const CORDILLERA_TILES: [(i32, i32); 2] = [(0, 0), (0, 1)]; // green dirt/grass
const ILOCOS_TILES: [(i32, i32); 3] = [(2, 1), (3, 1), (2, 2)]; // brown sand
const BICOL_TILES: [(i32, i32); 3] = [(4, 0), (5, 0), (4, 1)]; // red volcanic
const CALABARZON_TILES: [(i32, i32); 2] = [(3, 0), (2, 0)]; // sandstone

/// Added to the world seed for the biome noise so biome regions do not line up
/// with the altitude noise. Deterministic: same world seed -> same biome layout.
pub const BIOME_SEED_OFFSET: i32 = 7919;

/// Low frequency so biomes form large continuous regions.
pub const BIOME_FREQUENCY: f32 = 0.002;

/// Added to the world seed for the variant noise. Different from BIOME_SEED_OFFSET
/// so variant patches do not align with biome-region boundaries.
pub const VARIANT_SEED_OFFSET: i32 = 31337;

/// Medium frequency so variants form small patches (nearby cells share a variant).
pub const VARIANT_FREQUENCY: f32 = 0.05;

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
    /// This biome's same-family ground-tile variants on [`BIOME_SOURCE_ID`].
    #[inline]
    fn variants(self) -> &'static [(i32, i32)] {
        match self {
            Biome::Ncr => &NCR_TILES,
            Biome::Cordillera => &CORDILLERA_TILES,
            Biome::Ilocos => &ILOCOS_TILES,
            Biome::Bicol => &BICOL_TILES,
            Biome::Calabarzon => &CALABARZON_TILES,
        }
    }

    /// Pick this biome's ground tile for a cell from a raw variant-noise sample
    /// (FastNoiseLite range ~ -1.0..1.0). The normalized value is split into equal
    /// bands, one per variant (2 variants -> split at 0.5; 3 -> at 0.33/0.66).
    #[inline]
    pub fn variant_tile(self, variant_noise: f32) -> (i32, i32) {
        let variants = self.variants();
        let v = (variant_noise + 1.0) * 0.5; // normalize -1.0..1.0 -> 0.0..1.0
        // Saturating float->int cast keeps idx in range even if v is <0 or >=1.
        let idx = ((v * variants.len() as f32) as usize).min(variants.len() - 1);
        variants[idx]
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
