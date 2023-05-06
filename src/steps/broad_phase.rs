//! The broad phase is responsible for collecting potential collision pairs into the [`BroadCollisionPairs`] resource using simple AABB intersection checks. This reduces the number of required precise collision checks. See [`BroadPhasePlugin`].

use crate::prelude::*;
use bevy::{prelude::*, utils::StableHashSet};

/// The `BroadPhasePlugin` is responsible for collecting potential collision pairs into the [`BroadCollisionPairs`] resource using simple AABB intersection checks.
///
/// The broad phase speeds up collision detection, as the number of accurate collision checks is greatly reduced.
pub struct BroadPhasePlugin;

impl Plugin for BroadPhasePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BroadCollisionPairs>()
            .init_resource::<AabbIntervals>();

        let xpbd_schedule = app
            .get_schedule_mut(XpbdSchedule)
            .expect("add xpbd schedule first");

        xpbd_schedule.add_systems(
            (
                remove_old_aabb_intervals,
                update_aabb_intervals,
                add_new_aabb_intervals,
                collect_collision_pairs,
            )
                .chain()
                .in_set(PhysicsSet::BroadPhase),
        );
    }
}

/// A list of entity pairs for potential collisions collected during the broad phase.
#[derive(Resource, Default, Debug)]
pub struct BroadCollisionPairs(pub Vec<(Entity, Entity)>);

/// The [`ColliderAabb`]s sorted along an axis by their extents.
#[derive(Resource, Default)]
struct AabbIntervals(Vec<(Entity, ColliderAabb)>);

/// Updates [`AabbIntervals`] to keep them in sync with the [`ColliderAabb`]s.
fn update_aabb_intervals(aabbs: Query<&ColliderAabb>, mut intervals: ResMut<AabbIntervals>) {
    for (ent, aabb) in intervals.0.iter_mut() {
        *aabb = *aabbs.get(*ent).unwrap();
    }
}

/// Adds new [`ColliderAabb`]s to [`AabbIntervals`].
fn add_new_aabb_intervals(
    aabbs: Query<(Entity, &ColliderAabb), Added<ColliderAabb>>,
    mut intervals: ResMut<AabbIntervals>,
) {
    let aabbs = aabbs.iter().map(|(ent, aabb)| (ent, *aabb));
    intervals.0.extend(aabbs);
}

/// Removes old [`ColliderAabb`]s from [`AabbIntervals`].
fn remove_old_aabb_intervals(
    mut removals: RemovedComponents<ColliderAabb>,
    mut intervals: ResMut<AabbIntervals>,
) {
    let removed = removals.iter().collect::<StableHashSet<Entity>>();
    intervals.0.retain(|(ent, _)| !removed.contains(ent));
}

/// Collects bodies that are potentially colliding.
fn collect_collision_pairs(
    intervals: ResMut<AabbIntervals>,
    mut broad_collision_pairs: ResMut<BroadCollisionPairs>,
) {
    sweep_and_prune(intervals, &mut broad_collision_pairs.0);
}

/// Sorts the entities by their minimum extents along an axis and collects the entity pairs that have intersecting AABBs.
///
/// Sweep and prune exploits temporal coherence, as bodies are unlikely to move significantly between two simulation steps. Insertion sort is used, as it is good at sorting nearly sorted lists efficiently.
fn sweep_and_prune(
    mut intervals: ResMut<AabbIntervals>,
    broad_collision_pairs: &mut Vec<(Entity, Entity)>,
) {
    // Sort bodies along the x-axis using insertion sort, a sorting algorithm great for sorting nearly sorted lists.
    insertion_sort(&mut intervals.0, |a, b| a.1.mins.x > b.1.mins.x);

    // Clear broad phase collisions from previous iteration.
    broad_collision_pairs.clear();

    // Find potential collisions by checking for AABB intersections along all axes.
    for (i, (ent1, aabb1)) in intervals.0.iter().enumerate() {
        for (ent2, aabb2) in intervals.0.iter().skip(i + 1) {
            // x doesn't intersect
            if aabb2.mins.x > aabb1.maxs.x {
                break;
            }

            // y doesn't intersect
            if aabb1.mins.y > aabb2.maxs.y || aabb1.maxs.y < aabb2.mins.y {
                continue;
            }

            #[cfg(feature = "3d")]
            // z doesn't intersect
            if aabb1.mins.z > aabb2.maxs.z || aabb1.maxs.z < aabb2.mins.z {
                continue;
            }

            broad_collision_pairs.push((*ent1, *ent2));
        }
    }
}

/// Sorts a list iteratively using comparisons. In an ascending sort order, when a smaller value is encountered, it is moved lower in the list until it is larger than the item before it.
///
/// This is relatively slow for large lists, but very efficient in cases where the list is already mostly sorted.
fn insertion_sort<T>(items: &mut Vec<T>, comparison: fn(&T, &T) -> bool) {
    for i in 1..items.len() {
        let mut j = i;
        while j > 0 && comparison(&items[j - 1], &items[j]) {
            items.swap(j - 1, j);
            j -= 1;
        }
    }
}
