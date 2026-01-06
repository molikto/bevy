//! Stage system

use core::marker::PhantomData;

use crate::{
    component::ComponentId, prelude::Resource, storage::SparseSetIndex
};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;


/// Used to mark a stage by type
#[derive(Debug, Clone, Copy, Resource)]
pub struct StageMarker<T: Send +Sync + 'static> {
    /// The stage.
    pub stage: Stage,
    _marker: PhantomData<T>,
}
impl <T: Send +Sync + 'static> StageMarker<T> {
    /// Creates a new `StageMarker` wrapping the given stage.
    pub fn new(stage: Stage) -> Self {
        Self {
            stage,
            _marker: PhantomData,
        }
    }
}

/// World can have a sequence of stages
/// Component added in previous stages are immutable
/// Component added in later stages are not visible
/// You can only modify/remove components added in the current stage
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Hash, PartialEq, Clone)
)]
pub struct Stage {
    stage: u16,
}

impl SparseSetIndex for Stage {
    fn sparse_set_index(&self) -> usize {
        self.stage as usize
    }

    fn get_sparse_set_index(value: usize) -> Self {
        Self {
            stage: value as u16,
        }
    }
}

impl Stage {
    /// The maximum relative age for a change stage.
    /// The value of this is equal to [`MAX_CHANGE_AGE`].
    ///
    /// Since change detection will not work for any stages older than this,
    /// stages are periodically scanned to ensure their relative values are below this.
    pub const MAX: Self = Self::new(u16::MAX);

    /// Creates a new [`Stage`] wrapping the given value.
    #[inline]
    pub const fn new(stage: u16) -> Self {
        Self { stage }
    }

    /// Returns the next stage.
    pub fn next(&self) -> Self {
        Self {
            stage: self.stage.strict_add(1),
        }
    }

    /// Gets the value of this change stage.
    #[inline]
    pub const fn get(self) -> u16 {
        self.stage
    }

    /// Sets the value of this change stage.
    #[inline]
    pub fn set(&mut self, stage: u16) {
        self.stage = stage;
    }
}

/// A unique identifier for a component at a specific stage.
#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct StageComponentId {
    /// The component ID.
    pub component_id: ComponentId,
    /// The stage.
    pub stage: Stage,
}


impl StageComponentId {
    /// Creates a new `StageComponentId` with the given component ID and stage.
    pub const fn new(component_id: ComponentId, stage: Stage) -> Self {
        Self {
            component_id,
            stage,
        }
    }
}


impl ComponentId {
    /// Creates a `StageComponentId` at the given stage.
    pub fn at_stage(self, stage: Stage) -> StageComponentId {
        StageComponentId {
            component_id: self,
            stage,
        }
    }
}

/// Note:
/// We need to manually add each system to a separate schedue.
/// because systems get the stage when prepared.
/// and the entire schedule will have same stage.
#[cfg(test)]
mod tests {

    use crate::{query::With, stage::{Stage, StageMarker}, system::AtStage};

    use super::super::prelude::*;
    use std::*;
    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    #[test]
    #[should_panic]
    fn test_cannot_remove_component_from_previous_stage() {
        let mut world = World::new();
        let start_stage = Stage::new(1);
        world.set_stage(start_stage);

        let _entity = world.spawn(A).id();

        // Advance stage
        world.set_stage(start_stage.next());

        fn remove_a(mut commands: Commands, q: Query<Entity, With<A>>) {
            for e in q.iter() {
                commands.entity(e).remove::<A>();
            }
        }

        let _ = world.run_system_cached(remove_a).unwrap();
    }

    #[test]
    #[should_panic]
    fn cannot_move_children_from_previous_stage() {
        let mut world = World::new();
        let start_stage = Stage::new(1);
        world.set_stage(start_stage);
        let parent = world.spawn(A).id();
        let child = world.spawn((A, B, ChildOf(parent))).id();
        let _new_parent = world.spawn(B).id();

        fn move_children(
            mut commands: Commands,
            child_q: Single<Entity, (With<ChildOf>, With<B>)>,
            new_parent_q: Single<Entity, (Without<A>, With<B>)>,
        ) {
            println!("moving child");
            let child = child_q.into_inner();
            let new_parent = new_parent_q.into_inner();
            commands.entity(new_parent).add_child(child);
        }
        world.set_stage(start_stage.next());
        let _ = world.run_system_cached(move_children).unwrap();
    }

    #[test]
    fn test_can_remove_component_from_same_stage() {
        let mut world = World::new();
        let start_stage = Stage::new(1);
        world.set_stage(start_stage);
        let _entity = world.spawn(A).id();
        fn remove_a(mut commands: Commands, q: Query<Entity, With<A>>) {
            for e in q.iter() {
                commands.entity(e).remove::<A>();
            }
        }
        let _ = world.run_system_cached(remove_a).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_cannot_modify_component_from_previous_stage() {
        let mut world = World::new();
        let start_stage = Stage::new(1);
        world.set_stage(start_stage);
        let _entity = world.spawn(A).id();
        // Advance stage
        world.set_stage(start_stage.next());
        fn modify_a(mut q: Query<&mut A>) {
            for mut a in q.iter_mut() {
                let _ = &mut *a;
            }
        }
        let _ = world.run_system_cached(modify_a).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_cannot_modify_component_from_later_stage_by_world_ops() {
        let mut world = World::new();
        let start_stage = Stage::new(1);
        world.set_stage(start_stage);
        let entity = world.spawn(A).id();
        // Advance stage
        world.set_stage(start_stage.next().next());
        world.entity_mut(entity).get_mut::<A>().unwrap();
    }
    
    #[test]
    fn test_can_modify_component_from_same_stage() {
        let mut world = World::new();
        let start_stage = Stage::new(1);
        world.set_stage(start_stage);
        let _entity = world.spawn(A).id();
        fn modify_a(mut q: Query<&mut A>) {
            for mut a in q.iter_mut() {
                let _ = &mut *a;
            }
        }
        let _ = world.run_system_cached(modify_a).unwrap();
    }

    #[test]
    fn simple_stage_test() {
        let mut world = World::new();
        let _entity = world.spawn((A, B)).id();
        let _another_entity = world.spawn(A).id();
        let _b_only = world.spawn(B).id();
        world.flush();

        fn t1(qa: Query<&A>, qb: Query<&B>, qab: Query<Entity, (With<A>, With<B>)>, stage: Stage)  {
            println!("running t1 at stage {:?}", stage);
            let res = (qa.iter().count(), qb.iter().count(), qab.iter().count());
            assert_eq!(res, (2, 2, 1));
        }
        let mut s1 = Schedule::default();
        s1.add_systems(t1);

        fn t2(mut commands: Commands, qa: Query<(Entity, &A), Without<B>>, stage: Stage)  {
            println!("Running t2 at stage {:?}", stage);
            let Some(_) = qa.iter().next() else {
                // this is second tick
                return
            };
            commands.entity(qa.iter().next().unwrap().0).insert(B);
            let res = qa.iter().count();
            assert_eq!(res, 1);
        }
        let mut s2 = Schedule::default();
        s2.add_systems(t2);


        fn t3(qb: Query<&B>, stage: Stage)  {
            println!("running t3 at stage {:?}", stage);
            let res = qb.iter().count();
            assert_eq!(res, 3);
        }
        let mut s3 = Schedule::default();
        s3.add_systems(t3);

        let mut run_once = || {
            let start_stage = Stage::new(1);
            world.set_stage(start_stage.next());
            s1.run(&mut world);
            world.set_stage(start_stage.next().next());
            s2.run(&mut world);
            world.set_stage(start_stage.next().next().next());
            s3.run(&mut world);

        };
        run_once();
        // even at second run
        // we still see consistent prints.
        // because a system at stage N only sees changes up to stage N-1
        run_once();
    }

    #[test]
    fn insert_order_visibility() {
        // in first run, insert A at stage 2, and verify A isn't present before that.
        // in second run, insert A at stage 1, and verify A is present.
        let mut world = World::new();

        // add at stage 2
        fn stage2_without_a(mut commands: Commands,
            query: Query<&A>,
             stage: Stage) {
            println!("Running stage2_without_a at stage {:?}", stage);
            let count = query.iter().count();
            assert_eq!(count, 0);
            commands.spawn((A,));
        }
        world.set_stage(Stage::new(2));
        let mut schedule_stage2_without_a = Schedule::default();
        schedule_stage2_without_a.add_systems(stage2_without_a);
        schedule_stage2_without_a.run(&mut world);

        // at stage 1, assert A is not present yet, then insert A
        fn stage1_insert_a(mut commands: Commands,
            query: Query<&A>,
             stage: Stage) {
                assert_eq!(query.iter().count(), 0);
            println!("Running stage1_insert_a at stage {:?}", stage);
            commands.spawn((A,));
        }
        world.set_stage(Stage::new(1));
        let mut schedule_stage1_insert_a = Schedule::default();
        schedule_stage1_insert_a.add_systems(stage1_insert_a);
        schedule_stage1_insert_a.run(&mut world);

        // at stage 3, verify both inserts of A are visible, total 2 A's, add again
        fn stage3_with_a(query: Query<&A>, stage: Stage, mut command: Commands) {
            println!("Running stage2_with_a at stage {:?}", stage);
            let count = query.iter().count();
            command.spawn((A,));
            assert_eq!(count, 2);
        }
        world.set_stage(Stage::new(3));
        let mut schedule_stage3_with_a = Schedule::default();
        schedule_stage3_with_a.add_systems(stage3_with_a);
        // here both inserts of A at stage 1 and 2 are visible
        schedule_stage3_with_a.run(&mut world);

        // at stage 2, verify we see 2 A's
        fn check_at_stage_2(query: Query<&A>, stage: Stage) {
            println!("Running check_at_stage_2 at stage {:?}", stage);
            let count = query.iter().count();
            assert_eq!(count, 2);
        }
        world.set_stage(Stage::new(2));
        let mut schedule_check_at_stage_2 = Schedule::default();
        schedule_check_at_stage_2.add_systems(check_at_stage_2);
        schedule_check_at_stage_2.run(&mut world);

        fn final_check(query: Query<&A>, stage: Stage) {
            println!("Running final_check at stage {:?}", stage);
            let count = query.iter().count();
            assert_eq!(count, 3);
        }
        world.set_stage(Stage::new(3));
        let mut schedule_final_check = Schedule::default();
        schedule_final_check.add_systems(final_check);
        schedule_final_check.run(&mut world);
    }

    #[derive(Component)]
    struct ValueComponent(i32);
    #[test]
    fn insert_overwrite_later_stage() {
        let mut world = World::new();
        let entity = world.spawn(A).id();

        fn insert_a_at_stage_2(mut commands: Commands,
            query: Query<Entity, (With<A>, Without<ValueComponent>)>) {
            let entity= query.iter().next().unwrap();
            commands.entity(entity).insert(ValueComponent(10));
        }
        world.set_stage(Stage::new(2));
        let mut schedule_insert_a_at_stage_2 = Schedule::default();
        schedule_insert_a_at_stage_2.add_systems(insert_a_at_stage_2);
        schedule_insert_a_at_stage_2.run(&mut world);

        fn insert_a_at_stage_1(mut commands: Commands,
            query: Query<Entity, (With<A>, Without<ValueComponent>)>) {
            let entity= query.iter().next().unwrap();

            commands.entity(entity).insert(ValueComponent(0));
        }
        world.set_stage(Stage::new(1));
        let mut schedule_insert_a_at_stage_1 = Schedule::default();
        schedule_insert_a_at_stage_1.add_systems(insert_a_at_stage_1);
        schedule_insert_a_at_stage_1.run(&mut world);

        fn check_value_component(query: Query<&ValueComponent>, stage: Stage) {
            println!("Running check_value_component at stage {:?}", stage);
            let value_comp = query.iter().next().unwrap();
            // the insert at stage 2 should override the one at stage 1
            assert_eq!(value_comp.0, 0);
        }

        world.set_stage(Stage::new(3));
        let mut schedule_check_value_component = Schedule::default();
        schedule_check_value_component.add_systems(check_value_component);
        schedule_check_value_component.run(&mut world);


        fn assert_is_gone(
            query: Query<Entity, (With<A>, Without<ValueComponent>)>,
        ) {
                assert!(query.iter().next().is_none());
            
        }
        world.set_stage(Stage::new(2));
        let mut schedule_assert_is_gone = Schedule::default();
        schedule_assert_is_gone.add_systems(assert_is_gone);
        schedule_assert_is_gone.run(&mut world);

        let ccount = world.entity(entity).archetype().component_count();
        assert_eq!(ccount, 2); // A and ValueComponent

        // NOT working yet. becuase outiside of querys, we don't have stage-aware get_mut.
        // world.set_stage(Stage::new(0));
        // let ccount = world.entity(entity).archetype().component_count();
        // assert_eq!(ccount, 1); // A and ValueComponent
    }

    #[test]
    fn change_at_later_stage_is_not_detected() {
        let mut world = World::new();
        let _entity = world.spawn(A).id();

        fn insert_value_component_at_stage_2(mut commands: Commands,
            query: Query<Entity, (With<A>, Without<ValueComponent>)>) {
            let entity= query.iter().next().unwrap();
            commands.entity(entity).insert(ValueComponent(10));
        }
        world.set_stage(Stage::new(2));
        let mut schedule_insert_value_component_at_stage_2 = Schedule::default();
        schedule_insert_value_component_at_stage_2.add_systems(insert_value_component_at_stage_2);
        schedule_insert_value_component_at_stage_2.run(&mut world);

        fn check_change_detection(
            query: Query<&ValueComponent, Changed<ValueComponent>>,
            stage: Stage,
        ) {
            assert_eq!(query.iter().count(), 1, "at stage {:?}", stage);
        }

        world.set_stage(Stage::new(3));
        let mut schedule_check_change_detection = Schedule::default();
        schedule_check_change_detection.add_systems(check_change_detection);
        schedule_check_change_detection.run(&mut world);

        fn check_no_change_detection(
            query: Query<&ValueComponent, Changed<ValueComponent>>,
            stage: Stage,
        ) {
            assert_eq!(query.iter().count(), 0, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(1));
        let mut schedule_check_no_change_detection = Schedule::default();
        schedule_check_no_change_detection.add_systems(check_no_change_detection);
        schedule_check_no_change_detection.run(&mut world);
    }

    #[test]
    fn component_removal_at_later_stage_not_detected() {
        let mut world = World::new();
        world.set_stage(Stage::new(2));
        let _entity = world.spawn((A, ValueComponent(5))).id();
        fn remove_value_component_at_stage_2(mut commands: Commands,
            query: Query<Entity, (With<A>, With<ValueComponent>)>) {
            let entity= query.iter().next().unwrap();
            commands.entity(entity).remove::<ValueComponent>();
        }
        world.set_stage(Stage::new(2));
        let mut schedule_remove_value_component_at_stage_2 = Schedule::default();
        schedule_remove_value_component_at_stage_2.add_systems(remove_value_component_at_stage_2);
        schedule_remove_value_component_at_stage_2.run(&mut world);

        fn check_removal_detection(
            removed_components: RemovedComponents<ValueComponent>,
            stage: Stage,
        ) {
            assert_eq!(removed_components.len(), 1, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(3));
        let mut schedule_check_removal_detection = Schedule::default();
        schedule_check_removal_detection.add_systems(check_removal_detection);
        schedule_check_removal_detection.run(&mut world);

        fn check_no_removal_detection(
            removed_components: RemovedComponents<ValueComponent>,
            stage: Stage,
        ) {
            assert_eq!(removed_components.len(), 0, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(1));
        let mut schedule_check_no_removal_detection = Schedule::default();
        schedule_check_no_removal_detection.add_systems(check_no_removal_detection);
        schedule_check_no_removal_detection.run(&mut world);
    }

    #[test]
    fn test_staged_commands() {
        let mut world = World::new();
        struct M;
        world.insert_resource(StageMarker::<M>::new(Stage::new(1)));
        fn insert_a_at_stage_m(
            mut commands: AtStage<M, Commands>,
            _query: Query<Entity, With<A>>,
            _stage: Stage,
        ) {
            commands.spawn(A);
        }
        let mut schedule_insert_a_at_stage_m = Schedule::default();
        schedule_insert_a_at_stage_m.add_systems(insert_a_at_stage_m);
        schedule_insert_a_at_stage_m.run(&mut world);

        fn count_a_at_0(
            query: Query<&A>,
            stage: Stage,
        ) {
            assert_eq!(query.iter().count(), 0, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(0));
        let mut schedule_count_a_at_0 = Schedule::default();
        schedule_count_a_at_0.add_systems(count_a_at_0);
        schedule_count_a_at_0.run(&mut world);

        fn count_a_at_1(
            query: Query<&A>,
            stage: Stage,
        ) {
            assert_eq!(query.iter().count(), 1, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(1));
        let mut schedule_count_a_at_1 = Schedule::default();
        schedule_count_a_at_1.add_systems(count_a_at_1);
        schedule_count_a_at_1.run(&mut world);
    }

    #[test]
    fn move_to_stage() {
        let mut world = World::new();
        let _entity = world.spawn(A).id();
        // changes all data from stage 0 to stage 2
        // it rewrites the archetypes and component storages
        // the new stage must be empty, otherwise we cannot merge safely
        world.rewrite_all_stage(Stage::new(0), Stage::new(2));
        fn check_at_stage_1(
            query: Query<&A>,
            stage: Stage,
        ) {
            assert_eq!(query.iter().count(), 0, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(1));
        let mut schedule_check_at_stage_1 = Schedule::default();
        schedule_check_at_stage_1.add_systems(check_at_stage_1);
        schedule_check_at_stage_1.run(&mut world);

        fn check_at_stage_2(
            query: Query<&A>,
            stage: Stage,
        ) {
            assert_eq!(query.iter().count(), 1, "at stage {:?}", stage);
        }
        world.set_stage(Stage::new(2));
        let mut schedule_check_at_stage_2 = Schedule::default();
        schedule_check_at_stage_2.add_systems(check_at_stage_2);
        schedule_check_at_stage_2.run(&mut world);
    }
}
