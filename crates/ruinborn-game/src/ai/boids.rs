//! Flocking steering — Craig Reynolds' classic boids algorithm
//! (Separation, Alignment, Cohesion).
//!
//! Reference: <https://www.red3d.com/cwr/boids/> — derived structurally
//! from <https://github.com/SuboptimalEng/boids> (Unity, CC BY-NC-SA 4.0).
//! Only the algorithm is reused; no code is copied.
//!
//! ## Design contract
//!
//! - **Pure function**: takes a snapshot slice + the index of the boid
//!   we're computing steering for, returns a 2-D offset vector. Does
//!   not touch game state.
//! - **Same-kind only**: zombies don't align with skeletons. Different
//!   archetype ids are simply ignored as neighbours.
//! - **Same-zone only**: cross-zone neighbours are ignored (zones are
//!   independent simulation buckets in Phase 4 already).
//! - **Magnitude-bounded**: the returned offset is clamped to length
//!   ≤ `max_force` so it can be combined with a desired-direction
//!   vector without runaway acceleration.
//!
//! ## Usage from [`crate::combat::tick_enemies`]
//!
//! ```ignore
//! let snapshot: Vec<BoidSample> = build_snapshot(enemies);
//! for i in 0..enemies.len() {
//!     // … existing AI work …
//!     let (sx, sz) = boids::flocking_offset(i, &snapshot, &PARAMS_DEFAULT);
//!     // mix into desired direction, normalise, apply move_speed
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::world::ZoneId;

/// Per-tick read-only view of one enemy's position + recent velocity.
/// We snapshot every enemy *before* the mutation loop so each boid
/// can sample its neighbours without invalidating `&mut [Enemy]`.
///
/// Phase 6: dropped `Copy` because [`ZoneId`] is now an `Arc<str>`
/// newtype. All consumers iterate by reference, so the only impact
/// is that struct-update syntax (`..base`) requires `base.clone()`
/// at the call site — see test helpers below.
#[derive(Debug, Clone)]
pub struct BoidSample {
    pub x: f64,
    pub z: f64,
    /// Velocity of the *previous* tick — alignment uses this as a
    /// proxy for the neighbour's current heading. Zero is a fine
    /// default for fresh spawns.
    pub vx: f64,
    pub vz: f64,
    pub zone: ZoneId,
    /// Archetype id index — boids only flock with the *same* kind.
    /// We use a `u32` slot rather than the string id so the inner
    /// loop is a cheap integer compare instead of `String::eq`.
    pub kind_slot: u32,
    pub alive: bool,
}

/// Tunable steering weights. Defaults match the classic Reynolds
/// presentation with separation slightly dominant — enough to prevent
/// pile-ups on a single attack target without breaking the pack.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoidParams {
    /// Neighbours within this radius contribute to alignment + cohesion.
    pub view_radius: f64,
    /// Neighbours within this *smaller* radius push us away (separation).
    pub separation_radius: f64,
    pub separation_weight: f64,
    pub alignment_weight: f64,
    pub cohesion_weight: f64,
    /// Hard clamp on the returned offset's magnitude.
    pub max_force: f64,
}

impl Default for BoidParams {
    fn default() -> Self {
        Self {
            view_radius: 4.0,
            separation_radius: 1.5,
            separation_weight: 1.6,
            alignment_weight: 1.0,
            cohesion_weight: 0.8,
            max_force: 0.06,
        }
    }
}

/// Compute the combined separation + alignment + cohesion offset for
/// the boid at `self_idx`. Returns `(0.0, 0.0)` when the boid is dead,
/// has no same-kind neighbours, or `self_idx` is out of bounds.
pub fn flocking_offset(self_idx: usize, samples: &[BoidSample], p: &BoidParams) -> (f64, f64) {
    let Some(me) = samples.get(self_idx) else {
        return (0.0, 0.0);
    };
    if !me.alive {
        return (0.0, 0.0);
    }

    let view_sq = p.view_radius * p.view_radius;
    let sep_sq = p.separation_radius * p.separation_radius;

    let mut sep_x = 0.0_f64;
    let mut sep_z = 0.0_f64;
    let mut sum_vx = 0.0_f64;
    let mut sum_vz = 0.0_f64;
    let mut sum_cx = 0.0_f64;
    let mut sum_cz = 0.0_f64;
    let mut view_count: u32 = 0;
    let mut sep_count: u32 = 0;

    for (i, n) in samples.iter().enumerate() {
        if i == self_idx {
            continue;
        }
        if !n.alive {
            continue;
        }
        if n.zone != me.zone {
            continue;
        }
        if n.kind_slot != me.kind_slot {
            continue;
        }
        let dx = n.x - me.x;
        let dz = n.z - me.z;
        let dist_sq = dx * dx + dz * dz;
        if dist_sq > view_sq {
            continue;
        }

        // Alignment + cohesion contribute over the full view radius.
        sum_vx += n.vx;
        sum_vz += n.vz;
        sum_cx += n.x;
        sum_cz += n.z;
        view_count += 1;

        // Separation only inside the inner radius — and inverse-weighted
        // so neighbours pressed up against us push hardest. We use
        // `dist + epsilon` to avoid divide-by-zero on perfect overlap.
        if dist_sq < sep_sq {
            let dist = dist_sq.sqrt().max(1e-4);
            let inv = 1.0 / dist;
            sep_x -= (dx / dist) * inv;
            sep_z -= (dz / dist) * inv;
            sep_count += 1;
        }
    }

    if view_count == 0 && sep_count == 0 {
        return (0.0, 0.0);
    }

    // Average alignment heading.
    let (align_x, align_z) = if view_count > 0 {
        let n = view_count as f64;
        normalize(sum_vx / n, sum_vz / n)
    } else {
        (0.0, 0.0)
    };

    // Cohesion: vector toward the neighbour centroid.
    let (coh_x, coh_z) = if view_count > 0 {
        let n = view_count as f64;
        let cx = sum_cx / n - me.x;
        let cz = sum_cz / n - me.z;
        normalize(cx, cz)
    } else {
        (0.0, 0.0)
    };

    // Separation already has a sane magnitude from inverse-distance
    // weighting; normalising it would discard "pressure intensity"
    // info, so we leave it as-is and just clamp the final sum.
    let mut ox = p.separation_weight * sep_x
        + p.alignment_weight * align_x
        + p.cohesion_weight * coh_x;
    let mut oz = p.separation_weight * sep_z
        + p.alignment_weight * align_z
        + p.cohesion_weight * coh_z;

    let mag = (ox * ox + oz * oz).sqrt();
    if mag > p.max_force {
        let s = p.max_force / mag;
        ox *= s;
        oz *= s;
    }

    (ox, oz)
}

fn normalize(x: f64, z: f64) -> (f64, f64) {
    let mag = (x * x + z * z).sqrt();
    if mag < 1e-6 {
        (0.0, 0.0)
    } else {
        (x / mag, z / mag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(x: f64, z: f64, vx: f64, vz: f64, kind: u32) -> BoidSample {
        BoidSample {
            x,
            z,
            vx,
            vz,
            zone: ZoneId::new("blood_moor"),
            kind_slot: kind,
            alive: true,
        }
    }

    #[test]
    fn lone_boid_has_no_steering() {
        let s = vec![sample(0.0, 0.0, 0.0, 0.0, 1)];
        assert_eq!(flocking_offset(0, &s, &BoidParams::default()), (0.0, 0.0));
    }

    #[test]
    fn dead_boid_has_no_steering() {
        let mut s = vec![sample(0.0, 0.0, 0.0, 0.0, 1), sample(1.0, 0.0, 0.0, 0.0, 1)];
        s[0].alive = false;
        assert_eq!(flocking_offset(0, &s, &BoidParams::default()), (0.0, 0.0));
    }

    #[test]
    fn separation_pushes_away_from_close_neighbour() {
        // Neighbour to the +x side, very close → we should be pushed -x.
        let s = vec![
            sample(0.0, 0.0, 0.0, 0.0, 1),
            sample(0.5, 0.0, 0.0, 0.0, 1),
        ];
        let (ox, _) = flocking_offset(0, &s, &BoidParams::default());
        assert!(ox < 0.0, "expected negative-x push, got {ox}");
    }

    #[test]
    fn different_kinds_do_not_flock() {
        // Neighbour at the separation distance but a different kind →
        // must produce zero steering.
        let s = vec![
            sample(0.0, 0.0, 0.0, 0.0, 1),
            sample(0.5, 0.0, 1.0, 0.0, 9),
        ];
        let (ox, oz) = flocking_offset(0, &s, &BoidParams::default());
        assert_eq!((ox, oz), (0.0, 0.0));
    }

    #[test]
    fn cross_zone_neighbours_are_ignored() {
        let s = vec![
            sample(0.0, 0.0, 0.0, 0.0, 1),
            BoidSample {
                zone: ZoneId::new("rogue_encampment"),
                ..sample(0.5, 0.0, 1.0, 0.0, 1)
            },
        ];
        let (ox, oz) = flocking_offset(0, &s, &BoidParams::default());
        assert_eq!((ox, oz), (0.0, 0.0));
    }

    #[test]
    fn alignment_pulls_toward_neighbour_heading() {
        // Two neighbours moving +x, just outside separation but inside view.
        // Pure alignment should tilt our offset's x positive.
        let p = BoidParams {
            separation_weight: 0.0, // isolate alignment
            cohesion_weight: 0.0,
            ..BoidParams::default()
        };
        let s = vec![
            sample(0.0, 0.0, 0.0, 0.0, 1),
            sample(2.5, 0.0, 1.0, 0.0, 1),
            sample(2.5, 1.0, 1.0, 0.0, 1),
        ];
        let (ox, _) = flocking_offset(0, &s, &p);
        assert!(ox > 0.0, "expected +x alignment pull, got {ox}");
    }

    #[test]
    fn cohesion_pulls_toward_centroid() {
        let p = BoidParams {
            separation_weight: 0.0,
            alignment_weight: 0.0,
            ..BoidParams::default()
        };
        // Two neighbours clustered at +x → centroid at +x → cohesion +x.
        let s = vec![
            sample(0.0, 0.0, 0.0, 0.0, 1),
            sample(2.5, 0.5, 0.0, 0.0, 1),
            sample(2.5, -0.5, 0.0, 0.0, 1),
        ];
        let (ox, _) = flocking_offset(0, &s, &p);
        assert!(ox > 0.0, "expected +x cohesion pull, got {ox}");
    }

    #[test]
    fn output_is_clamped_to_max_force() {
        // Two neighbours stacked on top of us → separation magnitude
        // would explode; clamp must kick in.
        let s = vec![
            sample(0.0, 0.0, 0.0, 0.0, 1),
            sample(0.01, 0.0, 0.0, 0.0, 1),
            sample(0.0, 0.01, 0.0, 0.0, 1),
        ];
        let p = BoidParams::default();
        let (ox, oz) = flocking_offset(0, &s, &p);
        let mag = (ox * ox + oz * oz).sqrt();
        assert!(
            mag <= p.max_force + 1e-9,
            "magnitude {mag} exceeded max_force {}",
            p.max_force
        );
    }
}
