#[cfg(test)]
mod tests {
    use super::super::prelude::*;
    use std::*;

    #[test]
    fn simple_stage_test() {
        let mut world = World::new();
        let entity = world.spawn(()).id();
        let child = world.spawn((ChildOf(entity),)).id();
        fn testing_system(test: Query<(Entity, &ChildOf)>) {
            println!("Running testing_system");
        }
        println!("start");
        let mut schedule = Schedule::default();
        schedule.add_systems(testing_system);
        println!("registered");
        schedule.run(&mut world);
        println!("ran");
        schedule.run(&mut world);
    }
}