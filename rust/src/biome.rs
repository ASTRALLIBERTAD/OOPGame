//! Luzon biome system (Stages 1-3).
//!
//! Stage 1: the single world is split into 5 large Luzon biomes. A low-frequency
//! biome noise (seeded deterministically from the world seed) is sampled per land
//! cell and bucketed into one of 5 value bands, one band per biome.
//!
//! Stage 2: each biome owns 2-3 same-family ground-tile variants. After the biome
//! is picked, a medium-frequency variant noise is sampled at the same cell so
//! nearby cells tend to share a variant, forming small patchy clusters rather than
//! per-cell scatter.
//!
//! Stage 3 (anchor-based clustering): each biome owns single-tile decorations chosen to
//! match its color and the real Luzon place it is named after -- urban clutter for NCR
//! (Metro Manila), lush green bushes for the Cordillera (Baguio/Banaue forests), dry
//! yellow scrub for Ilocos (Paoay dunes), red volcanic mushrooms/fruit for Bicol
//! (Mayon/chili country), and farm greenery for CALABARZON. They grow in discrete
//! clusters on a layer above the ground:
//!   - sparse "anchor" cells are picked deterministically across the land (a low
//!     per-cell hash chance, scaled by the biome's density and by altitude so peaks get
//!     fewer anchors);
//!   - each anchor is a focal "parent" plant whose hash fixes the cluster's SPECIES, so
//!     one kind grows together (a stand of bushes here, a stand of mushrooms there);
//!   - every land cell finds its nearest anchor within CLUSTER_RADIUS and decorates by
//!     distance -- the biggest tile at the anchor, medium tiles close in, the smallest
//!     tiles thinning out toward the edge, and bare ground beyond.
//! Each cell scans its neighbourhood in WORLD coordinates, so clusters span chunk
//! borders seamlessly. Everything is deterministic, so chunks reload identically. Water
//! is excluded (altitude gate). Multi-tile trees and collision come later.

/// Tile source id shared by every biome ground tile (world_tileset.png).
pub const BIOME_SOURCE_ID: i32 = 1;

// --- Stage 3.5 water tiles on BIOME_SOURCE_ID. ---
/// Deep water: water cells NOT touching land (unchanged). Atlas (0,11) = bright cyan.
pub const DEEP_WATER_TILE: (i32, i32) = (0, 11);
/// Shallow/shore water: water cells adjacent to land. (0,11) deep water is already the
/// lightest solid blue in the tileset, so the lighter "shallow" pick is the white foam
/// tile (0,9). Swap to (0,10) for a foamy-dotted look, or (0,13) for a darker rim.
pub const SHORE_WATER_TILE: (i32, i32) = (0, 9);

// --- Biome ground-tile variants: atlas (x, y) on BIOME_SOURCE_ID, same color
// family per biome. First entry doubles as the Stage 1 base tile. Edit to retune. ---
const NCR_TILES: [(i32, i32); 2] = [(8, 0), (8, 1)]; // black rock
const CORDILLERA_TILES: [(i32, i32); 2] = [(0, 0), (0, 1)]; // green dirt/grass
const ILOCOS_TILES: [(i32, i32); 3] = [(2, 1), (3, 1), (2, 2)]; // brown sand
const BICOL_TILES: [(i32, i32); 3] = [(4, 0), (5, 0), (4, 1)]; // red volcanic
const CALABARZON_TILES: [(i32, i32); 2] = [(3, 0), (2, 0)]; // sandstone

// --- Per-biome decorations, grouped into SPECIES, each size-ordered big -> small.
// The type noise picks a species (one kind clusters together); the maturity noise picks
// the SIZE within it (big at maturity peaks, smaller around). atlas (x, y) on
// BIOME_SOURCE_ID. Edit freely. ---
// NCR = Metro Manila (urban, black rock): man-made clutter, single-size (no growth).
const NCR_SPECIES: &[&[(i32, i32)]] = &[&[(1, 0)], &[(7, 4)], &[(8, 4)]]; // rock, crate, sign
// Cordillera = mountain forests / Baguio "City of Flowers": lush green, flowers as the
// small ones. Each bush row is round (big) -> medium -> flat/flowered (small).
const CORDILLERA_SPECIES: &[&[(i32, i32)]] = &[
    &[(1, 3), (1, 4), (1, 6)], // light green bush: round, medium, flowered
    &[(6, 6), (6, 7), (6, 8)], // dark green bush: round, medium, flat
    &[(7, 8), (8, 8)],         // green mushroom: big, small
];
// Ilocos = dry windswept coast / Paoay dunes: dry yellow scrub + the odd rock.
const ILOCOS_SPECIES: &[&[(i32, i32)]] = &[
    &[(5, 6), (5, 7), (5, 8)], // yellow bush: round, medium, flat
    &[(7, 6), (8, 6)],         // yellow mushroom: big, small
    &[(1, 1)],                 // rock (single size)
];
// Bicol = Mayon volcano / chili country: red & dark-pink mushrooms + the odd fruit.
const BICOL_SPECIES: &[&[(i32, i32)]] = &[
    &[(7, 5), (8, 5)], // red mushroom: big, small
    &[(7, 7), (8, 7)], // dark-pink mushroom: big, small
    &[(4, 8)],         // fruit (single size)
];
// CALABARZON = agricultural lowlands: mixed green/yellow bushes + the odd fruit.
const CALABARZON_SPECIES: &[&[(i32, i32)]] = &[
    &[(6, 6), (6, 7), (6, 8)], // dark green bush: round, medium, flat
    &[(5, 6), (5, 7), (5, 8)], // yellow bush: round, medium, flat
    &[(4, 8)],                 // fruit (single size)
];

// Per-biome decoration density multipliers (relative lushness of each place). The
// spawn chance is scaled by this AND by the global DECORATION_DENSITY master knob, so
// the Cordillera is the lushest while NCR (a city) stays the sparsest. Values > 1.0
// are allowed (denser). Edit freely.
const NCR_DENSITY: f32 = 0.55; // dense city -> sparsest, but not empty
const CORDILLERA_DENSITY: f32 = 1.10; // mountain forests -> lushest
const ILOCOS_DENSITY: f32 = 0.80; // dry coast/dunes -> a touch drier than the rest
const BICOL_DENSITY: f32 = 1.00; // volcanic + tropical -> dense
const CALABARZON_DENSITY: f32 = 0.90; // farm lowlands -> fairly full

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

/// MASTER decoration density knob: scales the anchor chance (and thus all decorations)
/// at once. Raise for a fuller, busier world; lower for a sparser one. The per-biome
/// `*_DENSITY` set the relative mix; this sets the overall level -- the single number
/// to turn if the whole world feels too empty or too busy.
pub const DECORATION_DENSITY: f32 = 1.0;

/// Base probability that a land cell is a cluster ANCHOR (a focal "parent" plant). Kept
/// low so anchors are sparse and well separated, with bare ground between clusters.
/// Scaled per cell by DECORATION_DENSITY and the anchor's biome density.
pub const ANCHOR_CHANCE: f32 = 0.02;

/// Upper bound on any biome's density multiplier, used only for a cheap early reject in
/// the anchor scan: a cell whose anchor hash is at/above ANCHOR_COARSE_CHANCE cannot be
/// an anchor, so we skip sampling altitude/biome there. Keep >= the largest `*_DENSITY`.
pub const ANCHOR_DENSITY_CEIL: f32 = 2.0;

/// Cheap coarse cutoff for the anchor scan = the largest possible effective anchor
/// chance. If a cell's anchor hash is at/above this, it is definitely not an anchor.
pub const ANCHOR_COARSE_CHANCE: f32 = ANCHOR_CHANCE * DECORATION_DENSITY * ANCHOR_DENSITY_CEIL;

/// Cluster radius in cells: companions spawn within this distance of an anchor; beyond
/// it the ground is bare. Bigger = larger stands. Drives the size tiers and thinning.
pub const CLUSTER_RADIUS: i32 = 3;

/// How fast a cluster thins with distance from its anchor: spawn probability is
/// (1 - dist/CLUSTER_RADIUS)^this. Higher = denser core / sparser fringe; 1.0 = linear.
pub const CLUSTER_THINNING_STEEPNESS: f32 = 1.5;

/// Altitude bias for anchors: their chance is multiplied by
/// clamp01(1 - ALTITUDE_BIAS_STRENGTH * altitude), so fewer anchors sit on high peaks.
/// 0 = no bias; higher = stronger highland suppression.
pub const ALTITUDE_BIAS_STRENGTH: f32 = 0.5;

/// Added to the world seed for the anchor-existence hash. Deterministic, no randi().
pub const ANCHOR_SEED_OFFSET: i32 = 314_159;

/// Added to the world seed for the per-anchor SPECIES hash (which species a cluster is).
pub const ANCHOR_SPECIES_SEED_OFFSET: i32 = 271_009;

/// Added to the world seed for the per-cell thinning roll inside a cluster.
pub const DECOR_SEED_OFFSET: i32 = 90_001;

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

    /// This biome's decoration SPECIES on [`BIOME_SOURCE_ID`]; each inner slice is one
    /// species, size-ordered big -> small.
    #[inline]
    fn decorations(self) -> &'static [&'static [(i32, i32)]] {
        match self {
            Biome::Ncr => NCR_SPECIES,
            Biome::Cordillera => CORDILLERA_SPECIES,
            Biome::Ilocos => ILOCOS_SPECIES,
            Biome::Bicol => BICOL_SPECIES,
            Biome::Calabarzon => CALABARZON_SPECIES,
        }
    }

    /// Per-biome decoration density multiplier (#4): density depends on the biome's
    /// real-world character -- lush places decorate heavily, urban/arid ones sparsely.
    #[inline]
    fn decoration_density(self) -> f32 {
        match self {
            Biome::Ncr => NCR_DENSITY,
            Biome::Cordillera => CORDILLERA_DENSITY,
            Biome::Ilocos => ILOCOS_DENSITY,
            Biome::Bicol => BICOL_DENSITY,
            Biome::Calabarzon => CALABARZON_DENSITY,
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

/// Deterministic per-cell hash: mixes a world cell and seed into a well-distributed
/// `u32`. Pure function (no `randi()`), so a cell hashes identically on every reload.
#[inline]
fn cell_hash(x: i32, y: i32, seed: i32) -> u32 {
    // Combine coordinates and seed, then avalanche (xorshift-multiply finalizer).
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1);
    h ^= (y as u32).wrapping_mul(0x85EB_CA77);
    h ^= (seed as u32).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    h
}

/// Normalized anchor-existence hash for a cell, in 0.0..1.0. Cheap; the caller compares
/// it against the anchor chance (after the cheap [`ANCHOR_COARSE_CHANCE`] reject).
#[inline]
pub fn anchor_hash_roll(cx: i32, cy: i32, world_seed: i32) -> f32 {
    cell_hash(cx, cy, world_seed.wrapping_add(ANCHOR_SEED_OFFSET)) as f32 / u32::MAX as f32
}

/// Effective anchor chance at a confirmed land cell: the base chance scaled by the
/// master knob, the cell's biome density, and an altitude factor (fewer anchors on
/// peaks). A cell is an anchor iff its [`anchor_hash_roll`] is below this.
#[inline]
pub fn anchor_effective_chance(biome: Biome, altitude: f32) -> f32 {
    let altitude_factor = (1.0 - ALTITUDE_BIAS_STRENGTH * altitude).clamp(0.0, 1.0);
    ANCHOR_CHANCE * DECORATION_DENSITY * biome.decoration_density() * altitude_factor
}

/// Decoration tile for a land cell that lies `dist` cells from the given anchor, or
/// `None` if it thins out. The anchor (its position + biome) fixes the cluster's
/// species, so the whole cluster is one kind; the distance picks the size (biggest at
/// the anchor, smaller toward the edge) and, via the per-cell hash, the thinning that
/// fades the cluster outward. Deterministic, so chunks reload identically.
#[inline]
pub fn cluster_decoration(
    anchor_biome: Biome,
    anchor_x: i32,
    anchor_y: i32,
    dist: f32,
    cell_x: i32,
    cell_y: i32,
    world_seed: i32,
) -> Option<(i32, i32)> {
    let species = anchor_biome.decorations();
    if species.is_empty() {
        return None;
    }
    // Species comes from the ANCHOR, so the whole cluster is one kind.
    let sp_idx = cell_hash(anchor_x, anchor_y, world_seed.wrapping_add(ANCHOR_SPECIES_SEED_OFFSET))
        as usize
        % species.len();
    let kind = species[sp_idx];
    if kind.is_empty() {
        return None;
    }
    let t = (dist / CLUSTER_RADIUS as f32).clamp(0.0, 1.0); // 0 at anchor, 1 at the edge
    // Size: biggest at the anchor (t=0 -> index 0), smallest toward the edge. Only the
    // anchor cell itself has dist < 1, so it is the unique biggest plant.
    let size_idx = ((t * kind.len() as f32) as usize).min(kind.len() - 1);
    // Thinning: dense at the anchor (always placed), sparse at the fringe.
    let thin_prob = (1.0 - t).powf(CLUSTER_THINNING_STEEPNESS);
    let thin_roll =
        cell_hash(cell_x, cell_y, world_seed.wrapping_add(DECOR_SEED_OFFSET)) as f32 / u32::MAX as f32;
    if thin_roll >= thin_prob {
        return None;
    }
    Some(kind[size_idx])
}
