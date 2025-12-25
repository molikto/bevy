#[cfg(test)]
mod tests {
    use crate::{change_detection::Tick, system::SystemChangeTick};

    use super::super::prelude::*;
    use std::*;
    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    #[test]
    fn simple_stage_test() {

        let mut world = World::new();
        let entity = world.spawn((A, B)).id();
        let another_entity = world.spawn(A).id();

        fn t1(qa: Query<&A>, qb: Query<&B>, tick: SystemChangeTick) {
            println!("running t1 at tick {:?}", tick);
            println!("A count: {}, B count: {}", qa.iter().count(), qb.iter().count());
        }
        let mut s_t1 = Schedule::default();
        s_t1.add_systems(t1);

        fn t2(mut commands: Commands, qa: Query<(Entity, &A), Without<B>>, tick: SystemChangeTick) {
            println!("Running t2 at tick {:?}", tick);
            println!("A without B count: {}", qa.iter().count());
            commands.entity(qa.iter().next().unwrap().0).insert(B);
        }
        let mut s_t2 = Schedule::default();
        s_t2.add_systems(t2);

        fn t3(qb: Query<&B>, tick: SystemChangeTick) {
            println!("running t3 at tick {:?}", tick);
            println!("B count: {}", qb.iter().count());
        }
        let mut s_t3 = Schedule::default();
        s_t3.add_systems(t3);

        let mut run_once = || {
            let start_tick = Tick::new(1);
            println!("start: {:?}", world.change_tick());

            world.set_change_tick(start_tick.next());
            s_t1.run(&mut world);
            // A count: 2, B count: 1

            world.set_change_tick(start_tick.next().next());
            s_t2.run(&mut world);
            // A without B count: 1

            world.set_change_tick(start_tick.next().next().next());
            s_t3.run(&mut world);
            // B count: 2
        };
        run_once();
        // even at second run
        // we still see consistent prints.
        // because a system at tick N only sees changes up to tick N-1
        run_once();
    }
}
