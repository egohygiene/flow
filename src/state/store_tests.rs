use super::{CommitFault, RunStore};
use crate::{RunPlan, RunStepStatus, StateError};
use std::fs;

#[test]
fn interruptions_at_each_commit_boundary_preserve_an_honest_reopen_result() {
    let parent = std::env::temp_dir().join(format!("flow-atomic-commit-{}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    for (index, phase) in [
        CommitFault::Created,
        CommitFault::Synced,
        CommitFault::Renamed,
    ]
    .into_iter()
    .enumerate()
    {
        let workspace = parent.join(index.to_string());
        let plan: RunPlan = serde_json::from_str(include_str!(
            "../../contracts/examples/run-plan.v1.example.json"
        ))
        .unwrap();
        let mut store = RunStore::create(&workspace, plan).unwrap();
        let original = fs::read(workspace.join("00000000000000000000.json")).unwrap();
        store.fault = Some(phase);
        assert!(matches!(
            store.cancel_pending("step:inspect"),
            Err(StateError::Io { .. })
        ));
        assert_eq!(store.state().steps[0].status, RunStepStatus::Pending);
        assert!(matches!(
            store.cancel_pending("step:inspect"),
            Err(StateError::ReopenRequired)
        ));
        drop(store);
        assert_eq!(
            fs::read(workspace.join("00000000000000000000.json")).unwrap(),
            original
        );
        if phase == CommitFault::Renamed {
            let reopened = RunStore::open(&workspace).unwrap();
            assert_eq!(reopened.state().steps[0].status, RunStepStatus::Cancelled);
            assert_eq!(reopened.state().sequence, 1);
        } else {
            assert!(matches!(
                RunStore::open(&workspace),
                Err(StateError::IncompleteWrite)
            ));
            assert!(workspace.join("snapshot.pending").is_file());
        }
    }
    fs::remove_dir_all(parent).unwrap();
}
