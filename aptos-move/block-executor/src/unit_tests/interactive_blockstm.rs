// Copyright © Aptos Foundation

use crate::{
    blockstm_providers::interactive_blockstm::InteractiveBlockStmProvider,
    scheduler::{DependencyResult, Scheduler, SchedulerTask},
};
use std::sync::Arc;

#[test]
fn scheduler_first_wave() {
    let indices = vec![11, 22, 33, 55, 88, 99];
    let provider = Arc::new(InteractiveBlockStmProvider::new(indices.clone()));
    let s = Scheduler::new(provider);

    for &i in indices.iter().take(5) {
        // Nothing to validate.
        assert!(matches!(
            s.next_task(false),
            SchedulerTask::ExecutionTask((j, 0), None) if j == i
        ));
    }

    // validation index will not increase for the first execution wave
    // until the status becomes executed.
    assert!(matches!(
        s.finish_execution(11, 0, false),
        SchedulerTask::NoTask
    ));

    // Now we can validate version (11, 0).
    assert!(matches!(
        s.next_task(false),
        SchedulerTask::ValidationTask((11, 0), 0)
    ));

    assert!(matches!(
        s.next_task(false),
        SchedulerTask::ExecutionTask((99, 0), None)
    ));
    // Since (22, 0) is not EXECUTED, no validation tasks, and execution index
    // is already at the limit, so no tasks immediately available.
    assert!(matches!(s.next_task(false), SchedulerTask::NoTask));

    assert!(matches!(
        s.finish_execution(33, 0, false),
        SchedulerTask::NoTask
    ));
    // There should be no tasks, but finishing (22,0) should enable validating
    // (22, 0) then (33,0).
    assert!(matches!(s.next_task(false), SchedulerTask::NoTask));

    assert!(matches!(
        s.finish_execution(22, 0, false),
        SchedulerTask::NoTask
    ));
    assert!(matches!(
        s.next_task(false),
        SchedulerTask::ValidationTask((22, 0), 0)
    ));
    assert!(matches!(
        s.next_task(false),
        SchedulerTask::ValidationTask((33, 0), 0)
    ));
    assert!(matches!(s.next_task(false), SchedulerTask::NoTask));
}

#[test]
fn scheduler_dependency() {
    let indices = vec![2, 3, 5, 7, 11];
    let provider = Arc::new(InteractiveBlockStmProvider::new(indices.clone()));
    let s = Scheduler::new(provider);

    for i in indices {
        // Nothing to validate.
        assert!(matches!(
            s.next_task(false),
            SchedulerTask::ExecutionTask((j, 0), None) if j == i
        ));
    }

    // validation index will not increase for the first execution wave
    // until the status becomes executed.
    assert!(matches!(
        s.finish_execution(2, 0, false),
        SchedulerTask::NoTask
    ));
    // Now we can validate version (2, 0).
    assert!(matches!(
        s.next_task(false),
        SchedulerTask::ValidationTask((2, 0), 0)
    ));
    // Current status of 2 is executed - hence, no dependency added.
    assert!(matches!(
        s.wait_for_dependency(7, 2),
        DependencyResult::Resolved
    ));
    // Dependency added for transaction 11 on transaction 5.
    assert!(matches!(
        s.wait_for_dependency(11, 5),
        DependencyResult::Dependency(_)
    ));

    assert!(matches!(
        s.finish_execution(5, 0, false),
        SchedulerTask::NoTask
    ));

    // resumed task doesn't bump incarnation
    assert!(matches!(
        s.next_task(false),
        SchedulerTask::ExecutionTask((11, 0), Some(_))
    ));
}
